// TODO implement memory
//  * working RAM (WRAM)
//  * video RAM (VRAM)
//  * memory-mapped
//  * memory write modes 1 & 2

const ROM_SIZE: usize = 0x3FFF;
const WRAM_SIZE: usize = 0x1FFF;
const REGS_SIZE: usize = 0x7F;

pub struct Mmu {
    // TODO: emulate memory mapping
    rom: [u8; ROM_SIZE],
    wram: Vec<u8>,
    io_registers: Vec<u8>,
    interrupt_enable: bool,
}

impl Mmu {
    /// initializes memory sections
    pub fn new() -> Mmu {
        Mmu {
            rom: [0; ROM_SIZE],
            wram: vec![0; WRAM_SIZE],
            io_registers: vec![0; REGS_SIZE],
            interrupt_enable: false,
        }
    }

    /// TODO: reads a byte from the memory-mapped bus
    /// Impl
    pub fn read_byte(&self, address: u16) -> u8 {

        let address = address as usize;
        match address {
            0x0000..=0x3FFF => {    // fixed ROM. TODO reimplement as MBC
                self.rom[address]
            }
            0x4000..=0x7FFF => {
                // TODO implement MBC mapping
                unimplemented!("MBC read");
            }
            0x8000..=0x9FFF => {
                unimplemented!("VRAM read");
            }
            0xA000..=0xBFFF => {
                unimplemented!("Cartridge RAM read");
            }
            0xC000..=0xDFFF => { // TODO: test
                self.wram[address - 0xC000]
            }
            0xE000..=0xFDFF => {    // echos all WRAM r/w up to 0xDDFF
                unimplemented!("Echo RAM read");
            }
            0xFE00..=0xFE9F => {
                unimplemented!("Sprite attribute table read");
            }
            0xFEA0..=0xFEFF => 0, // unused memory area, returns 0
            0xFF00..=0xFF7F => {
                unimplemented!("I/O registers read");
            }
            0xFF80..=0xFFFE => {
                unimplemented!("High RAM read");
            }
            0xFFFF => {
                match self.interrupt_enable {
                    true => 1,
                    false => 0,
                }
            }
            // should never happen because input is u16
            _ => panic!("Invalid memory address read!")
        }
    }

    /// TODO: reads a word (2 bytes) from the memory
    pub fn read_word(&self, address: u16) -> u16 {
        0xF0F0
    }

    /// TODO: load game program to ROM
    pub fn load_rom(&mut self, data: Vec<u8>) -> () {
        for (i, instr) in data.iter().enumerate() {
            self.rom[i] = *instr;
        }
    }
}
