//! No MBC in cartrige. This provides direct rom access to ROM addresses
//! 0 through 0x7FFF for a total of 8 KB.

// TODO:
//  * tests for MbcNone reading and writing. Need to reference documentation
//  * MBC RAM

use super::*;

const ROM_SIZE_MBC_NONE: usize = 0x7FFF;

pub struct MbcNone {
    rom: [u8; ROM_SIZE_MBC_NONE],
}

impl MbcNone {
    pub fn new() -> MbcNone {
        MbcNone {
            rom: [0; ROM_SIZE_MBC_NONE],
        }
    }
}

impl MemoryBankController for MbcNone {
    /// Read a byte from the cartridge ROM
    fn read_byte(&self, address: u16) -> Result<u8, MBCError> {
        let address = address as usize;

        if address >= ROM_SIZE_MBC_NONE {
            return Err(MBCError::ROMAccessOutOfRange);
        }
        // other checks?

        Ok(self.rom[address])
    }

    /// Write a byte to the cartridge ROM
    fn write_byte(&mut self, address: u16, value: u8) -> Result<(), MBCError> {
        let address = address as usize;

        if address >= ROM_SIZE_MBC_NONE {
            return Err(MBCError::ROMAccessOutOfRange);
        }

        self.rom[address] = value;
        Ok(())
    }
}

#[cfg(test)]
mod tests {}
