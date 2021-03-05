// TODO implement memory
//  * working RAM (WRAM)
//  * video RAM (VRAM)
//  * memory-mapped
//  * memory write modes 1 & 2

pub struct MMU {
    // TODO: emulate memory mapping
}
impl MMU {
    /// initializes memory sections
    pub fn new() -> MMU {
        MMU {}
    }

    /// TODO: reads a byte from the memory
    pub fn read_byte(&self, address: u16) -> u8 {
        0xF0
    }

    /// TODO: reads a word (2 bytes) from the memory
    pub fn read_word(&self, address: u16) -> u16 {
        0xF0F0
    }
}
