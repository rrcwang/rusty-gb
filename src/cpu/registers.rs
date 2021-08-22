//! Implements the CPU registers

use crate::utils::{bytes_to_word, word_to_bytes};
use std::fmt;

#[derive(Debug)]
pub struct Registers {
    // 8 bit registers
    a: u8,
    f: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    // 16 bit registers
    pub sp: u16,
    pub pc: u16,
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum Register8b {
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum Register16b {
    AF,
    BC,
    DE,
    HL,
    SP,
    // stack pointer
    PC, // program counter
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum Flag {
    Z,
    // zero flag
    N,
    // add/sub
    H,
    // half carry
    C, // carry flag
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            // TODO: initial register values from AntonioND/giibiiadvance, §3.2
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: 0,
            pc: 0,
        }
    }

    /// Sets the value in the specified the 8 bit CPU register
    pub fn set_r8(&mut self, reg: Register8b, value: u8) {
        match reg {
            Register8b::A => self.a = value,
            Register8b::F => self.f = value, // should never be written to directly?
            Register8b::B => self.b = value,
            Register8b::C => self.c = value,
            Register8b::D => self.d = value,
            Register8b::E => self.e = value,
            Register8b::H => self.h = value,
            Register8b::L => self.l = value,
        }
    }

    /// Retrieves the value stored in the specified 8 bit CPU register.
    ///
    /// # Arguments
    ///
    /// * `reg` - The 8 bit register to be read from.
    pub fn get_r8(&self, reg: Register8b) -> u8 {
        match reg {
            Register8b::A => self.a,
            Register8b::F => self.f,
            Register8b::B => self.b,
            Register8b::C => self.c,
            Register8b::D => self.d,
            Register8b::E => self.e,
            Register8b::H => self.h,
            Register8b::L => self.l,
        }
    }

    /// Sets the value in one of the 16 bit CPU registers
    ///
    /// # Arguments
    ///
    /// * `reg` - The register written to.
    /// * `value` - `u16` value written to the specified register.
    pub fn set_r16(&mut self, reg: Register16b, value: u16) {
        match reg {
            Register16b::AF => {
                let (x, y) = word_to_bytes(value);
                self.a = x;
                self.f = y & 0xF0; // mask, lower 4 bits should always be 0
            }
            Register16b::BC => {
                let (x, y) = word_to_bytes(value);
                self.b = x;
                self.c = y;
            }
            Register16b::DE => {
                let (x, y) = word_to_bytes(value);
                self.d = x;
                self.e = y;
            }
            Register16b::HL => {
                let (x, y) = word_to_bytes(value);
                self.h = x;
                self.l = y;
            }
            Register16b::SP => self.sp = value,
            Register16b::PC => self.pc = value,
        }
    }
    /// Retrieves the value stored in the specified 16 bit CPU register.
    ///
    /// # Arguments
    ///
    /// * `reg` - The 16 bit register to be read from.
    pub fn get_r16(&self, reg: Register16b) -> u16 {
        match reg {
            Register16b::AF => bytes_to_word(self.a, self.f),
            Register16b::BC => bytes_to_word(self.b, self.c),
            Register16b::DE => bytes_to_word(self.d, self.e),
            Register16b::HL => bytes_to_word(self.h, self.l),
            Register16b::SP => self.sp,
            Register16b::PC => self.pc,
        }
    }

    /// Sets desired flag in CPU register F to 1.
    ///
    /// # Arguments
    ///
    /// * `flag` - The flag to be set. One of `Flag::{Z, N, H, C}`.
    pub fn set_flag(&mut self, flag: Flag, value: bool) {
        let mask = match flag {
            Flag::Z => 0b_1000_0000,
            Flag::N => 0b_0100_0000,
            Flag::H => 0b_0010_0000,
            Flag::C => 0b_0001_0000,
        };

        match value {
            true => self.f |= mask,
            false => self.f &= !mask,
        }
    }

    /// Returns the valued stored in one of the CPU flags.
    ///
    /// # Arguments
    ///
    /// * `flag` - The flag to be read from. One of `Flag::{Z, N, H, C}`.
    ///
    /// # Return value
    ///
    /// * `bool` type where
    /// | return value  | flag state           |
    /// |:-------------:|:--------------------:|
    /// | `true`        | `register.flag == 1` |
    /// | `false`       | `register.flag == 0` |
    pub fn flag_value(&self, flag: Flag) -> bool {
        let bit: u8 = match flag {
            Flag::Z => (self.f & 0b_1000_0000) >> 7,
            Flag::N => (self.f & 0b_0100_0000) >> 6,
            Flag::H => (self.f & 0b_0010_0000) >> 5,
            Flag::C => (self.f & 0b_0001_0000) >> 4,
        };

        match bit {
            0 => false,
            1 => true,
            _ => panic!("Nonesense result returned for CPU flag! (neither 0/1)"),
        }
    }
}

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "A:{:02X} F:{:04b} \
            B:{:02X} C:{:02X} \
            D:{:02X} E:{:02X} \
            H:{:02X} L:{:02X} \
            PC:{:04X} \
            SP:{:04X}",
            self.a, self.f, self.b, self.c, self.d, self.e, self.h, self.l, self.pc, self.sp
        )
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;

    /// Test helpers module
    pub(crate) mod common {
        use super::super::*;

        pub const REGISTERS_8B: &[Register8b] = &[
            Register8b::A,
            Register8b::F,
            Register8b::B,
            Register8b::C,
            Register8b::D,
            Register8b::E,
            Register8b::H,
            Register8b::L,
        ];

        pub const REGISTERS_16B: &[Register16b; 6] = &[
            Register16b::AF,
            Register16b::BC,
            Register16b::DE,
            Register16b::HL,
            Register16b::SP,
            Register16b::PC,
        ];

        pub const FLAGS: &[Flag; 4] = &[Flag::Z, Flag::N, Flag::H, Flag::C];
    }

    #[test]
    fn register_r8_set_get() {
        let mut registers = Registers::new();

        for reg in common::REGISTERS_8B {
            registers.set_r8(*reg, 0x1)
        }
        for reg in common::REGISTERS_8B {
            assert_eq!(1u8, registers.get_r8(*reg))
        }
    }

    #[test]
    fn register_r8_init_zero_get() {
        let registers = Registers::new();

        for reg in common::REGISTERS_8B {
            assert_eq!(0u8, registers.get_r8(*reg));
        }
    }

    #[test]
    fn register_r16_init_zero_get() {
        let registers = Registers::new();

        for reg in common::REGISTERS_16B {
            assert_eq!(0u16, registers.get_r16(*reg));
        }
    }

    #[test]
    fn register_r16_set_get() {
        let mut registers = Registers::new();

        for reg in common::REGISTERS_16B {
            registers.set_r16(*reg, 0xF0F0u16)
        }

        for reg in common::REGISTERS_16B {
            assert_eq!(0xF0F0u16, registers.get_r16(*reg));
        }
    }

    #[test]
    fn register_r16_set_r8_get() {
        let mut registers = Registers::new();

        for reg_16b in common::REGISTERS_16B {
            registers.set_r16(*reg_16b, 0xF0F0u16)
        };

        for reg_8b in common::REGISTERS_8B {
            assert_eq!(0xF0u8, registers.get_r8(*reg_8b));
        }
    }

    #[test]
    fn register_r8_set_r16_get() {
        let mut registers = Registers::new();
        ;

        for reg_8b in common::REGISTERS_8B {
            registers.set_r8(*reg_8b, 0xF0u8);
        }

        for reg_16b in vec![
            Register16b::AF,
            Register16b::BC,
            Register16b::DE,
            Register16b::HL,
        ] {
            // AF, BC, DE, HL
            assert_eq!(0xF0F0u16, registers.get_r16(reg_16b));
        }

        for reg_16b in vec![Register16b::SP, Register16b::PC] {
            // SP, PC
            assert_eq!(0, registers.get_r16(reg_16b));
        }
    }

    #[test]
    fn register_flag_init_zero_get() {
        let registers = Registers::new();

        for flag in common::FLAGS {
            assert_eq!(false, registers.flag_value(*flag));
        }
    }

    #[test]
    fn register_flag_set_clear() {
        let mut registers = Registers::new();
        // set all flags and check for true
        for flag in common::FLAGS {
            registers.set_flag(*flag, true);
        }

        for flag in common::FLAGS {
            assert_eq!(true, registers.flag_value(*flag));
        }

        for flag in common::FLAGS {
            registers.set_flag(*flag, false);
        }

        for flag in common::FLAGS {
            assert_eq!(false, registers.flag_value(*flag));
        }
    }
}
