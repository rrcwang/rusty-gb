// library imports
use std::{error::Error, fmt};
// module imports
pub mod mbc_none;
// export MBC types to other modules at this level
pub use mbc_none::MbcNone;

/// Trait for memory bank controllers (MBCs). All MBCs should have the same interface provided through this trait.
///     Takes
pub trait MemoryBankController {
    ///
    fn read_byte(&self, address: u16) -> Result<u8, MBCError>;
    ///
    fn write_byte(&mut self, address: u16, value: u8) -> Result<(), MBCError>;
}

#[derive(Debug, Clone)]
pub enum MBCError {
    ROMAccessOutOfRange,
}

impl Error for MBCError {}
impl fmt::Display for MBCError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            MBCError::ROMAccessOutOfRange => write!(f, "Invalid ROM address access attempted!"),
        }
    }
}
