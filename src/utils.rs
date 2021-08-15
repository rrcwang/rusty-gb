//! Contains common utility functions used across the project.

/// Splits a 16-bit word into two 8-bit bytes
///
/// # Arguments
/// * `value` - u16
///
/// # Outputs
/// * `(high, low)` - split bytes `value`.
///
/// # Notes
/// Gameboy CPU is little-endian, so a word is laid out in memory as:
/// | address    | byte  |
/// | ---------- | ----- |
/// | a          | low   |
/// | a+1        | high  |
pub(crate) fn word_to_bytes(value: u16) -> (u8, u8) {
    let high: u8 = (value >> 8) as u8;
    let low: u8 = (value & 0x00FF) as u8;
    (high, low)
}

pub(crate) fn bytes_to_word(high_byte: u8, low_byte: u8) -> u16 {
    let high: u16 = (high_byte as u16) << 8;
    high + low_byte as u16
}
