use crate::{crc8, dimensions::{COLS, ROWS}};
use arrayvec::ArrayVec;
use generic_array::{
    typenum::{U4, U7},
    GenericArray,
};
use cortex_m::asm::nop;
use keyberon::matrix::PressedKeys;

pub const SOF: u8 = 1 << 7;
pub const TX_BUF_LEN: usize = 6; // Must be ROWS * COLS / 8 + 2, rounded up
pub const RX_BUF_LEN: usize = TX_BUF_LEN - 1; // Minus one because we don't store the SOF byte
pub const SCAN_LEN: usize = COLS * ROWS;

pub fn encode_scan(scan: &PressedKeys<U4, U7>) -> [u8; TX_BUF_LEN] {
    let mut buf = [0u8; TX_BUF_LEN];
    buf[0] = SOF;
    for (i, &pressed) in scan.0.iter().flat_map(|r| r.iter()).enumerate() {
        if pressed {
            buf[i / 7 + 1] |= 1 << (i % 7);
        } else {
            buf[i / 7 + 1] |= 0;
        }
    }
    let checksum = crc8::MAXIM.calc_buf(&buf[1..TX_BUF_LEN - 1]);
    buf[TX_BUF_LEN - 1] = checksum;
    buf
}

pub fn decode_scan(buf: &[u8; RX_BUF_LEN]) -> Option<PressedKeys<U4, U7>> {
    let actual_checksum = crc8::MAXIM.calc_buf(&buf[..RX_BUF_LEN - 1]);
    if actual_checksum == buf[RX_BUF_LEN - 1] {
        let flat_scan: ArrayVec<bool, SCAN_LEN> = buf[..RX_BUF_LEN - 1]
            .iter()
            .flat_map(|&b| (0..7).map(move |i| b & (1 << i) != 0))
            .collect();
        let iter = &mut flat_scan.into_iter();
        Some(PressedKeys(
            GenericArray::from_exact_iter(
                (0..ROWS).map(|_| GenericArray::from_exact_iter(iter.take(COLS)).unwrap()),
            )
            .unwrap(),
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use generic_array::arr;
    use generic_array::typenum::{U4,U7};

    #[test]
    fn test_encode() {
        let mut scan: PressedKeys<U4, U7> = PressedKeys(arr![GenericArray<bool, U7>;
            arr![bool; false, false, false, false, false, false, false],
            arr![bool; false, false, false, false, false, false, false],
            arr![bool; false, false, false, false, false, false, false],
            arr![bool; false, false, false, false, false, false, false],
        ]);
        assert_eq!([128, 0, 0, 0, 0, 0], encode_scan(&scan));
        scan.0[0][0] = true;
        assert_eq!([128, 0b001, 0b00, 0, 0, 143], encode_scan(&scan));
        scan.0[0][1] = true;
        assert_eq!([128, 0b011, 0b00, 0, 0, 136], encode_scan(&scan));
        scan.0[0][2] = true;
        assert_eq!([128, 0b111, 0b00, 0, 0, 134], encode_scan(&scan));
        scan.0[0][6] = true;
        assert_eq!([128, 0b1000111, 0b00, 0, 0, 102], encode_scan(&scan));
        scan.0[1][0] = true;
        assert_eq!([128, 0b1000111, 0b01, 0, 0, 205], encode_scan(&scan));
    }

    #[test]
    fn test_decode() {}
}
