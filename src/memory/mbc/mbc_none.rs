const ROM_SIZE: usize = 0x3FFF;

struct MbcNone {
    rom: [u8; ROM_SIZE],
}

impl MemoryAccess for MbcNone {
    fn read_byte(address: u16) -> u8 {
        0
    }
    fn write_byte(address: u16, value: u8) {
        
    }
}