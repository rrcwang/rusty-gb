//! Contains common utility functions used across the project.
pub fn word_to_bytes(value: u16) -> (u8, u8) {
    let high: u8 = (value >> 8) as u8;
    let low: u8 = (value & 0x00FF) as u8;
    (high, low)
}

pub fn bytes_to_word(high_byte: u8, low_byte: u8) -> u16 {
    let high: u16 = (high_byte as u16) << 8;
    high + low_byte as u16
}
