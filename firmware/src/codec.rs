use crate::{
    crc8,
    ROWS,
    COLS
};
use generic_array::{
    GenericArray,
    typenum::{U4, U7},
};
use arrayvec::ArrayVec;
use keyberon::matrix::PressedKeys;

pub const SOF: u8 = 1 << 7;
const BUF_LEN: usize = 6;

// pub async fn receive_scan<R: Read<u8>>(uart: &mut R) -> HalfScan {
//     loop {
//         match nb_to_future(|| uart.read()).await {
//             Ok(v) if v == SOF => {}
//             _ => continue,
//         }
//         let mut buf = [0u8; 4];
//         for i in 0..4 {
//             buf[i] = match nb_to_future(|| uart.read()).await {
//                 Ok(v) => v,
//                 Err(_) => continue,
//             };
//         }
//         let expected_checksum = match nb_to_future(|| uart.read()).await {
//             Ok(v) => v,
//             Err(_) => continue,
//         };
//         let actual_checksum = crc8::MAXIM.calc_buf(&buf);
//         if actual_checksum != expected_checksum {
//             continue;
//         }
//         // TODO: decode buf
//         return new_half_scan();
//     }
// }

pub fn encode_scan(scan: &PressedKeys<U4, U7>) -> [u8; BUF_LEN] {
    let mut buf = [0u8; BUF_LEN];
    buf[0] = SOF;
    for (i, &pressed) in scan.0.iter().flat_map(|r| r.iter()).enumerate() {
        buf[i / 7 + 1] |= if pressed { 1 << (i % 7) } else { 0 };
    }
    let checksum = crc8::MAXIM.calc_buf(&buf[1..BUF_LEN-1]);
    buf[BUF_LEN-1] = checksum;
    buf
}

pub fn decode_scan(buf: &[u8; BUF_LEN]) -> Option<PressedKeys<U4, U7>> {
    let actual_checksum = crc8::MAXIM.calc_buf(&buf[1..BUF_LEN-1]);
    if actual_checksum == buf[5] {
        let flat_scan: ArrayVec<[bool; COLS*ROWS]> = buf[1..BUF_LEN-1]
            .iter()
            .flat_map(|&b|
                (0..7).map(move |i| b & (1 << i) != 0))
            .collect();
        let iter = &mut flat_scan.into_iter();
        Some(PressedKeys(
            GenericArray::from_exact_iter((0..ROWS)
                .map(|_| GenericArray::from_exact_iter(iter.take(COLS)).unwrap())).unwrap()))
    } else {
        None
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::new_half_scan;

    #[test]
    fn test_encode() {
        let mut scan = new_half_scan();
        assert_eq!([128, 0, 0, 0, 0, 0], encode(scan));
        scan[0][0] = true;
        assert_eq!([128, 0b001, 0b00, 0, 0, 143], encode(scan));
        scan[0][1] = true;
        assert_eq!([128, 0b011, 0b00, 0, 0, 136], encode(scan));
        scan[0][2] = true;
        assert_eq!([128, 0b111, 0b00, 0, 0, 134], encode(scan));
        scan[0][6] = true;
        assert_eq!([128, 0b1000111, 0b00, 0, 0, 102], encode(scan));
        scan[1][0] = true;
        assert_eq!([128, 0b1000111, 0b01, 0, 0, 205], encode(scan));
    }

    #[test]
    fn test_decode() {}
}
