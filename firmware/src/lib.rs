#![cfg_attr(not(test), no_std)]

pub mod codec;
pub mod crc8;

pub const ROWS: usize = 4;
pub const COLS: usize = 7;
