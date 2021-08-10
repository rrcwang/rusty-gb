use crate::utils::*;
use std::boxed::Box;

// TODO implement memory
//  * working RAM (WRAM)
//  * video RAM (VRAM)
//  * memory-mapped
//  * memory write modes 1 & 2
mod mbc;

const WRAM_SIZE: usize = 0x1FFF;
const REGS_SIZE: usize = 0x7F;

pub struct Mmu {
    // TODO: emulate memory mapping
    wram: Vec<u8>,
    io_registers: Vec<u8>,
    interrupt_enable: bool,
    mbc: Box<dyn mbc::MemoryBankController + 'static>,
}

impl Mmu {
    /// initializes memory sections
    pub fn new() -> Mmu {
        Mmu {
            wram: vec![0; WRAM_SIZE],
            io_registers: vec![0; REGS_SIZE],
            interrupt_enable: false,
            mbc: Box::new(mbc::MbcNone::new()),
        }
    }

    /// TODO: reads a byte from the memory-mapped bus
    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF => {
                // fixed ROM
                match self.mbc.read_byte(address) {
                    Ok(byte) => byte,
                    Err(_) => panic!("Cartirdge ROM read error."), // handle this?
                }
            }
            0x8000..=0x9FFF => {
                unimplemented!("VRAM read");
            }
            0xA000..=0xBFFF => {
                unimplemented!("Cartridge RAM read");
            }
            0xC000..=0xDFFF => {
                // WRAM read
                // TODO: test
                self.wram[address as usize - 0xC000]
            }
            0xE000..=0xFDFF => {
                // echos all WRAM r/w up to 0xDDFF
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
            0xFFFF => match self.interrupt_enable {
                true => 1,
                false => 0,
            },
            // should never happen because input is u16
            _ => panic!("Invalid memory address read!"),
        }
    }

    /// TODO: reads a word (2 bytes) from the memory. Needs test
    pub fn read_word(&self, address: u16) -> u16 {
        let low = self.read_byte(address);
        let high = self.read_byte(address + 1);
        bytes_to_word(high, low)
    }

    // TODO: writes a byte to the memory-mapped bus
    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x7FFF => {
                // fixed ROM
                self.mbc.write_byte(address, value);
            }
            0xC000..=0xDFFF => {
                self.wram[address as usize - 0xC000] = value;
            }
            _ => {
                unimplemented!("Memory write")
            }
        };
    }

    // TODO:
    pub fn write_word(&mut self, address: u16, value: u16) {
        let (high, low) = word_to_bytes(value);
        self.write_byte(address, low);
        self.write_byte(address + 1, high);
    }

    /// TODO: load game program to ROM
    pub fn load_rom(&mut self, data: Vec<u8>) -> () {
        for (i, instr) in data.iter().enumerate() {
            if let Err(_) = self.mbc.write_byte(i as u16, *instr) {
                panic!("ROM loading error! Illegal write to ROM address 0x{:X}", i);
            }
        }
    }
}

#[cfg(test)]
mod tests;
