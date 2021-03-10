pub trait MemoryAccess {
    fn read_byte(address: u16) -> u8;
    fn write_byte(address: u16, value: u8);
}

