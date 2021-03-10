// TODO implement memory
//  * working RAM (WRAM)
//  * video RAM (VRAM)
//  * memory-mapped
//  * memory write modes 1 & 2

const ROM_SIZE: usize = 0x3FFF;
const WRAM_SIZE: usize = 0xFFF;
const REGS_SIZE: usize = 0x7F;

pub struct MMU {
    // TODO: emulate memory mapping
    rom: [u8; ROM_SIZE],
    wram: Vec<u8>,
    io_registers: Vec<u8>,
}

impl MMU {
    /// initializes memory sections
    pub fn new() -> MMU {
        MMU {
            rom: [0; ROM_SIZE],
            wram: vec![0; WRAM_SIZE],
            io_registers: vec![0; REGS_SIZE],
        }
    }

    /// TODO: reads a byte from the memory
    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => {
                // TODO implement MBCs
                self.rom[address as usize]
            }
            0xFEA0..=0xFEFF => 0, // unused memory area, returns 0
            _ => unimplemented!("Memory adress read"),
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
