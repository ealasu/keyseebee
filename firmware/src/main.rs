#![no_main]
#![no_std]

#[cfg(not(test))] use panic_halt as _;

pub use atsamd_hal as hal;
use atsamd_hal::{
    clock::GenericClockController,
    define_pins,
    gpio::{
        self, Floating, Input, IntoFunction, OpenDrain, Output, Pa10, Pa11, Pa27, PfC, Port,
        PullUp, PushPull,
    },
    prelude::*,
    sercom::{PadPin, Sercom0Pad2, Sercom0Pad3, UART0},
    target_device::{
        self,
        gclk::{clkctrl::GEN_A, genctrl::SRC_A},
        TC3,
    },
    timer::TimerCounter,
    usb::UsbBus,
};
use core::convert::Infallible;
use embedded_hal::digital::v2::{InputPin, OutputPin};

use generic_array::typenum::{U4, U7};
use keyberon::{
    action::{k, l, m, Action, Action::*},
    debounce::Debouncer,
    impl_heterogenous_array,
    key_code::KbHidReport,
    key_code::KeyCode,
    layout::{Event, Layout},
    matrix::{Matrix, PressedKeys},
};
use nb::block;
use rtic::app;
use usb_device::{
    bus::UsbBusAllocator,
    class::UsbClass,
    device::{UsbDevice, UsbDeviceState},
};
use stuff::{
    codec::{encode_scan, decode_scan, SOF, RX_BUF_LEN},
    layers::LAYERS,
};

trait ResultExt<T> {
    fn get(self) -> T;
}
impl<T> ResultExt<T> for Result<T, Infallible> {
    fn get(self) -> T {
        match self {
            Ok(v) => v,
            Err(e) => match e {},
        }
    }
}

//define_pins!(
//    /// Maps the pins to their arduino names and
//    /// the numbers printed on the board.
//    struct Pins,
//    target_device: target_device,
//
//    /// Serial RX, sercom0pad3
//    pin rx = a11,
//    /// Serial TX, sercom0pad2
//    pin tx = a10,
//    /// The red LED
//    pin red_led = a27,
//    /// The USB D- pad
//    pin usb_dm = a24,
//    /// The USB D+ pad
//    pin usb_dp = a25,
//    /// Grounded for left side, disconnected for right side.
//    pin is_left = a23, // SCL
//
//    pin row0 = a19, // 12
//    pin row1 = a18, // 10
//    pin row2 = a7, // 9
//    pin row3 = a15, // 5
//
//    pin col0 = a14, // 2
//    pin col1 = a9, // 3
//    pin col2 = a8, // 4
//    pin col3 = a5, // A4
//    pin col4 = a6, // 8
//    pin col5 = a16, // 11
//    pin col6 = a17, // 13
//);

pub struct Cols(
    atsamd_hal::gpio::Pa14<Input<PullUp>>,
    atsamd_hal::gpio::Pa9<Input<PullUp>>,
    atsamd_hal::gpio::Pa8<Input<PullUp>>,
    atsamd_hal::gpio::Pa5<Input<PullUp>>,
    atsamd_hal::gpio::Pa6<Input<PullUp>>,
    atsamd_hal::gpio::Pa16<Input<PullUp>>,
    atsamd_hal::gpio::Pa17<Input<PullUp>>,
);
impl_heterogenous_array! {
    Cols,
    dyn InputPin<Error = ()>,
    U7,
    [0, 1, 2, 3, 4, 5, 6]
}

pub struct Rows(
    atsamd_hal::gpio::Pa19<Output<PushPull>>,
    atsamd_hal::gpio::Pa18<Output<PushPull>>,
    atsamd_hal::gpio::Pa7<Output<PushPull>>,
    atsamd_hal::gpio::Pa15<Output<PushPull>>,
);
impl_heterogenous_array! {
    Rows,
    dyn OutputPin<Error = ()>,
    U4,
    [0, 1, 2, 3]
}

#[app(device = atsamd_hal::target_device, peripherals = true)]
const APP: () = {
    struct Resources {
        usb_dev: UsbDevice<'static, UsbBus>,
        usb_class: keyberon::Class<'static, UsbBus, ()>,
        matrix: Matrix<Cols, Rows>,
        debouncer: Debouncer<PressedKeys<U4, U7>>,
        other_debouncer: Debouncer<PressedKeys<U4, U7>>,
        layout: Layout,
        timer: TimerCounter<TC3>,
        rx: atsamd_hal::sercom::Rx0,
        tx: atsamd_hal::sercom::Tx0,
        led: Pa27<Output<OpenDrain>>,
    }

    #[init]
    fn init(mut c: init::Context) -> init::LateResources {
        static mut USB_BUS: Option<UsbBusAllocator<UsbBus>> = None;

        let mut clocks = GenericClockController::with_external_32kosc(
            c.device.GCLK,
            &mut c.device.PM,
            &mut c.device.SYSCTRL,
            &mut c.device.NVMCTRL,
        );
        clocks.configure_gclk_divider_and_source(GEN_A::GCLK2, 1, SRC_A::DFLL48M, false);
        let gclk0 = clocks.gclk0();
        let gclk2 = clocks
            .get_gclk(GEN_A::GCLK2)
            .expect("Could not get clock 2");

        let mut port = c.device.PORT.split();
        let mut matrix = Matrix::new(
            Cols(
                port.pa14.into_pull_up_input(&mut port.port),
                port.pa9.into_pull_up_input(&mut port.port),
                port.pa8.into_pull_up_input(&mut port.port),
                port.pa5.into_pull_up_input(&mut port.port),
                port.pa6.into_pull_up_input(&mut port.port),
                port.pa16.into_pull_up_input(&mut port.port),
                port.pa17.into_pull_up_input(&mut port.port),
            ),
            Rows(
                port.pa19.into_push_pull_output(&mut port.port),
                port.pa18.into_push_pull_output(&mut port.port),
                port.pa7.into_push_pull_output(&mut port.port),
                port.pa15.into_push_pull_output(&mut port.port),
            ),
        )
        .unwrap();

        // Enter bootloader if Escape key is pressed when keyboard is plugged in
        let mut init_layout = Layout::new(LAYERS);
        let scan = matrix.get().unwrap();
        for (i, j) in scan.iter_pressed() {
            init_layout.event(Event::Press(i as u8, (6 - j) as u8));
        }
        init_layout.tick();
        if init_layout.keycodes().any(|k| k == KeyCode::Escape ) {
            // Set special value in last word of RAM for UF2 bootloader to see
            unsafe { *((0x20000000 + (16 * 1024) - 4) as *mut u32) = 0xf01669ef; }
            // Then reset the CPU to enter the bootloader
            cortex_m::peripheral::SCB::sys_reset();
        }

        let usb_clock = &clocks.usb(&gclk0).unwrap();
        let usb_bus = {
            *USB_BUS = Some(UsbBusAllocator::new(UsbBus::new(
                usb_clock,
                &mut c.device.PM,
                port.pa24.into_function(&mut port.port),
                port.pa25.into_function(&mut port.port),
                c.device.USB,
            )));
            USB_BUS.as_ref().unwrap()
        };
        let usb_class = keyberon::new_class(usb_bus, ());
        let usb_dev = keyberon::new_device(usb_bus);


        let mut timer = TimerCounter::tc3_(
            &clocks.tcc2_tc3(&gclk0).unwrap(),
            c.device.TC3,
            &mut c.device.PM,
        );
        timer.start(1.ms());
        timer.enable_interrupt();

        let rx_pin: Sercom0Pad3<_> = port
            .pa11
            .into_pull_down_input(&mut port.port)
            .into_pad(&mut port.port);
        let tx_pin: Sercom0Pad2<_> = port
            .pa10
            .into_push_pull_output(&mut port.port)
            .into_pad(&mut port.port);

        let uart_clk = clocks
            .sercom0_core(&gclk2)
            .expect("Could not configure sercom0 core clock");
        let uart = UART0::new(
            &uart_clk,
            115200.hz(),
            c.device.SERCOM0,
            &mut c.device.PM,
            (rx_pin, tx_pin),
        );
        uart.enable_rxc_interrupt();
        let (rx, tx) = uart.split();

        let mut led = port.pa27.into_open_drain_output(&mut port.port);
        led.set_high().unwrap();


        init::LateResources {
            usb_dev,
            usb_class,
            timer,
            debouncer: Debouncer::new(PressedKeys::default(), PressedKeys::default(), 5),
            other_debouncer: Debouncer::new(PressedKeys::default(), PressedKeys::default(), 5),
            matrix,
            layout: Layout::new(LAYERS),
            rx,
            tx,
            led,
        }
    }

    #[task(binds = USB, priority = 4, resources = [usb_dev, usb_class])]
    fn usb_rx(c: usb_rx::Context) {
        if c.resources.usb_dev.poll(&mut [c.resources.usb_class]) {
            c.resources.usb_class.poll();
        }
    }

    #[task(binds = SERCOM0, priority = 3, spawn = [handle_uart_frame], resources = [rx])]
    fn rx(c: rx::Context) {
        static mut BUF: [u8; RX_BUF_LEN] = [0; RX_BUF_LEN];
        static mut BUF_POS: usize = 0;

        while let Ok(b) = c.resources.rx.read() {
            if b == SOF {
                *BUF_POS = 0;
            } else {
                BUF[*BUF_POS] = b;
                *BUF_POS += 1;
                if *BUF_POS == RX_BUF_LEN {
                    *BUF_POS = 0;
                    let _ = c.spawn.handle_uart_frame(*BUF); // TODO: report errors somehow
                }
            }
        }
    }

    #[task(priority = 2, capacity = 1, spawn = [handle_event], resources = [other_debouncer])]
    fn handle_uart_frame(mut c: handle_uart_frame::Context, buf: [u8; RX_BUF_LEN]) {
        if let Some(scan) = decode_scan(&buf) {
            for event in c.resources.other_debouncer.events(scan) {
                let event = event.transform(|i, j| (i, j + 7));
                c.spawn.handle_event(Some(event)).unwrap();
            }
        }
    }
 
    #[task(priority = 2, capacity = 8, resources = [
        usb_dev, usb_class, layout,
        led
        ])]
    fn handle_event(mut c: handle_event::Context, event: Option<Event>) {
        if let Some(event) = event {
            c.resources.layout.event(event);
            c.resources.led.toggle();
        };
        c.resources.layout.tick();
        let report: KbHidReport = c.resources.layout.keycodes().collect();
        if !c
            .resources
            .usb_class
            .lock(|k| k.device_mut().set_keyboard_report(report.clone()))
        {
            return;
        }
        if c.resources.usb_dev.lock(|d| d.state()) != UsbDeviceState::Configured {
            return;
        }
        while let Ok(0) = c.resources.usb_class.lock(|k| k.write(report.as_bytes())) {}
    }

    #[task(
        binds = TC3,
        priority = 1,
        spawn = [handle_event],
        resources = [matrix, debouncer, timer, tx],
    )]
    fn tick(c: tick::Context) {
        c.resources.timer.wait().ok();

        let scan = c.resources.matrix.get().unwrap();
        let buf = encode_scan(&scan);
        for &b in &buf {
            let _ = block!(c
                .resources
                .tx
                .write(b)
                .map_err(|_| nb::Error::<()>::WouldBlock));
        }

        for event in c.resources.debouncer.events(scan) {
            let event = event.transform(|i, j| (i, 6 - j));
            c.spawn.handle_event(Some(event)).unwrap();
        }
        c.spawn.handle_event(None).unwrap();
    }

    // Unused interrupts that will be used by RTIC for software tasks
    extern "C" {
        fn ADC();
        fn DAC();
    }
};

// #[inline(never)]
// #[panic_handler]
// fn panic(_info: &core::panic::PanicInfo) -> ! {
//     use core::sync::atomic::{self, Ordering};
//     let mut port = c.device.PORT.split();
//     let mut led = port.pa27.into_open_drain_output(&mut port.port);
//     loop {
//         led.set_high().unwrap();
//         for _ in 0..20_000_000 {
//             atomic::compiler_fence(Ordering::SeqCst);
//         }
//         led.set_low().unwrap();
//         for _ in 0..20_000_000 {
//             atomic::compiler_fence(Ordering::SeqCst);
//         }
//     }
// }