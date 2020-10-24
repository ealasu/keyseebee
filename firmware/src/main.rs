#![no_main]
#![no_std]

mod crc8;
mod codec;

// set the panic handler
use panic_halt as _;

pub use atsamd_hal as hal;
use atsamd_hal::{
    clock::GenericClockController,
    define_pins,
    gpio::{self, Floating, Input, IntoFunction, Output, Pa10, Pa11, Pa27, PfC, Port, PullUp, PushPull, OpenDrain},
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
use itertools::Itertools;

use generic_array::typenum::{U4, U7};
use keyberon::{
    action::{k, l, m, Action, Action::*},
    debounce::Debouncer,
    impl_heterogenous_array,
    key_code::KbHidReport,
    key_code::KeyCode::*,
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

pub const ROWS: usize = 4;
pub const COLS: usize = 7;

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

define_pins!(
    /// Maps the pins to their arduino names and
    /// the numbers printed on the board.
    struct Pins,
    target_device: target_device,

    /// Serial RX, sercom0pad3
    pin rx = a11,
    /// Serial TX, sercom0pad2
    pin tx = a10,
    /// The red LED
    pin red_led = a27,
    /// The USB D- pad
    pin usb_dm = a24,
    /// The USB D+ pad
    pin usb_dp = a25,
    /// Grounded for left side, disconnected for right side.
    pin is_left = a23, // SCL

    pin row0 = a19, // 12
    pin row1 = a18, // 10
    pin row2 = a7, // 9
    pin row3 = a15, // 5

    pin col0 = a14, // 2
    pin col1 = a9, // 3
    pin col2 = a8, // 4
    pin col3 = a5, // A4
    pin col4 = a6, // 8
    pin col5 = a16, // 11
    pin col6 = a17, // 13
);

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

const CUT: Action = m(&[LShift, Delete]);
const COPY: Action = m(&[LCtrl, Insert]);
const PASTE: Action = m(&[LShift, Insert]);
const L2_ENTER: Action = HoldTap {
    timeout: 140,
    hold: &l(2),
    tap: &k(Enter),
};
const L1_SP: Action = HoldTap {
    timeout: 200,
    hold: &l(1),
    tap: &k(Space),
};
const CSPACE: Action = m(&[LCtrl, Space]);
macro_rules! s {
    ($k:ident) => {
        m(&[LShift, $k])
    };
}
macro_rules! a {
    ($k:ident) => {
        m(&[RAlt, $k])
    };
}

#[rustfmt::skip]
pub static LAYERS: keyberon::layout::Layers = &[
    &[
        &[k(Tab),     k(Q), k(W),  k(E),    k(R), k(T),    k(Y),     k(U),    k(I),   k(O),    k(P),     k(LBracket)],
        &[k(RBracket),k(A), k(S),  k(D),    k(F), k(G),    k(H),     k(J),    k(K),   k(L),    k(SColon),k(Quote)   ],
        &[k(Equal),   k(Z), k(X),  k(C),    k(V), k(B),    k(N),     k(M),    k(Comma),k(Dot), k(Slash), k(Bslash)  ],
        &[Trans,      Trans,k(LGui),k(LAlt),L1_SP,k(LCtrl),k(RShift),L2_ENTER,k(RAlt),k(BSpace),Trans,   Trans      ],
    ], &[
        &[Trans,         k(Pause),Trans,     k(PScreen),Trans,    Trans,Trans,      Trans,  k(Delete),Trans,  Trans,   Trans ],
        &[Trans,         Trans,   k(NumLock),k(Insert), k(Escape),Trans,k(CapsLock),k(Left),k(Down),  k(Up),  k(Right),Trans ],
        &[k(NonUsBslash),k(Undo), CUT,       COPY,      PASTE,    Trans,Trans,      k(Home),k(PgDown),k(PgUp),k(End),  Trans ],
        &[Trans,         Trans,   Trans,     Trans,     Trans,    Trans,Trans,      Trans,  Trans,    Trans,  Trans,   Trans ],
    ], &[
        &[s!(Grave),s!(Kb1),s!(Kb2),s!(Kb3),s!(Kb4),s!(Kb5),s!(Kb6),s!(Kb7),s!(Kb8),s!(Kb9),s!(Kb0),s!(Minus)],
        &[ k(Grave), k(Kb1), k(Kb2), k(Kb3), k(Kb4), k(Kb5), k(Kb6), k(Kb7), k(Kb8), k(Kb9), k(Kb0), k(Minus)],
        &[a!(Grave),a!(Kb1),a!(Kb2),a!(Kb3),a!(Kb4),a!(Kb5),a!(Kb6),a!(Kb7),a!(Kb8),a!(Kb9),a!(Kb0),a!(Minus)],
        &[Trans,    Trans,  Trans,  Trans,  CSPACE, Trans,  Trans,  Trans,  Trans,  Trans,  Trans,  Trans    ],
    ], &[
        &[k(F1),k(F2),k(F3),k(F4),k(F5),k(F6),k(F7),k(F8),k(F9),k(F10),k(F11),k(F12)],
        &[Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans, Trans, Trans ],
        &[Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans, Trans, Trans ],
        &[Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans, Trans, Trans ],
    ],
];

#[app(device = atsamd_hal::target_device, peripherals = true)]
const APP: () = {
    struct Resources {
        usb_dev: UsbDevice<'static, UsbBus>,
        usb_class: keyberon::Class<'static, UsbBus, ()>,
        matrix: Matrix<Cols, Rows>,
        debouncer: Debouncer<PressedKeys<U4, U7>>,
        layout: Layout,
        timer: TimerCounter<TC3>,
        // tx: serial::Tx<hal::pac::USART1>,
        // rx: serial::Rx<hal::pac::USART1>,
        uart: UART0<Sercom0Pad3<Pa11<PfC>>, Sercom0Pad2<Pa10<PfC>>, (), ()>,
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
        let matrix = Matrix::new(
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

        let mut led = port.pa27.into_open_drain_output(&mut port.port);
        led.set_high().unwrap();

        init::LateResources {
            usb_dev,
            usb_class,
            timer,
            debouncer: Debouncer::new(PressedKeys::default(), PressedKeys::default(), 5),
            matrix,
            layout: Layout::new(LAYERS),
            uart,
            led,
        }
    }

    // TODO: was USART0
    #[task(binds = SERCOM0, priority = 2, spawn = [handle_event], resources = [uart, led])]
    fn rx(c: rx::Context) {
        static mut BUF: [u8; 6] = [0; 6];

        while let Ok(b) = c.resources.uart.read() {
            BUF.rotate_left(1);
            BUF[5] = b;
            if BUF[0] == codec::SOF {
                let scan = codec::decode_scan(&BUF);

                //if let Ok(event) = de(&BUF[..]) {
                    //c.resources.led.toggle();
                    //let event = event.transform(|i, j| (i, 11 - j));
                    //c.spawn.handle_event(Some(event)).unwrap();
                //}
            }
        }
    }

    #[task(binds = USB, priority = 4, resources = [usb_dev, usb_class])]
    fn usb_rx(c: usb_rx::Context) {
        if c.resources.usb_dev.poll(&mut [c.resources.usb_class]) {
            c.resources.usb_class.poll();
        }
    }

    #[task(priority = 3, capacity = 8, resources = [usb_dev, usb_class, layout])]
    fn handle_event(mut c: handle_event::Context, event: Option<Event>) {
        let report: KbHidReport = match event {
            None => c.resources.layout.tick().collect(),
            Some(e) => {
                c.resources.layout.event(e).collect()
            }
        };
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
        priority = 2,
        spawn = [handle_event],
        resources = [matrix, debouncer, timer, uart, led],
    )]
    fn tick(c: tick::Context) {
        c.resources.timer.wait().ok();

        let scan = c.resources.matrix.get().unwrap();
        let buf = codec::encode_scan(&scan);
        for &b in buf.iter() {
            let _ = block!(c
                .resources
                .uart
                .write(b)
                .map_err(|_| nb::Error::<()>::WouldBlock));
        }

        for event in c
            .resources
            .debouncer
            .events(scan)
        {
            c.resources.led.toggle();
            //for &b in &ser(event) {
                //let res = block!(c
                    //.resources
                    //.uart
                    //.write(b)
                    //.map_err(|_| nb::Error::<()>::WouldBlock));
                //let _: Result<_, u32> = res.map_err(|_| 1);
            //}
            c.spawn.handle_event(Some(event)).unwrap();
        }
        c.spawn.handle_event(None).unwrap();
    }

    extern "C" {
        fn ADC();
    }
};

// fn de(bytes: &[u8]) -> Result<Event, ()> {
//     match *bytes {
//         [b'P', i, j, b'\n'] => Ok(Event::Press(i, j)),
//         [b'R', i, j, b'\n'] => Ok(Event::Release(i, j)),
//         _ => Err(()),
//     }
// }
// fn ser(e: Event) -> [u8; 4] {
//     match e {
//         Event::Press(i, j) => [b'P', i, j, b'\n'],
//         Event::Release(i, j) => [b'R', i, j, b'\n'],
//     }
// }
