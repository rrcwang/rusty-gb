// registers.rs
// Implements the CPU registers

use std::fmt;

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
    sp: u16,
    pc: u16,
}

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

pub enum Register16b {
    AF,
    BC,
    DE,
    HL,
    SP, // stack pointer
    PC, // program counter
}

pub enum Flag {
    Z, // zero flag
    N, // add/sub
    H, // half carry
    C, // carry flag
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
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

    /// Sets the value in one of the 8 bit CPU registers
    ///
    /// # Arguments
    ///
    /// * `reg` - The register to be written to.
    /// * `value` - u8 value written to the specified register
    pub fn set_8b_reg(&mut self, reg: &Register8b, value: u8) {
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

    /// Retrieves value stored in one of the 8 bit CPU registers.
    ///
    /// # Arguments
    ///
    /// * `reg` - The register to be read from.
    ///
    /// # Return value
    ///
    /// `u8` type.
    pub fn get_8b_reg(&self, reg: &Register8b) -> u8 {
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
    /// * `reg` - The register to be written to.
    /// * `value` - u16 value written to the specified register
    ///
    /// # Return value
    ///
    /// `u16` type.
    pub fn set_16b_reg(&mut self, reg: &Register16b, value: u16) {
        match reg {
            Register16b::AF => {
                let (x, y) = Registers::split_high_low_bytes(value);
                self.a = x;
                self.f = y;
            }
            Register16b::BC => {
                let (x, y) = Registers::split_high_low_bytes(value);
                self.b = x;
                self.c = y;
            }
            Register16b::DE => {
                let (x, y) = Registers::split_high_low_bytes(value);
                self.d = x;
                self.e = y;
            }
            Register16b::HL => {
                let (x, y) = Registers::split_high_low_bytes(value);
                self.h = x;
                self.l = y;
            }
            Register16b::SP => self.sp = value,
            Register16b::PC => self.pc = value,
        }
    }
    /// Retrieves value stored in one of the 8 bit CPU registers.
    ///
    /// # Arguments
    ///
    /// * `reg` - The register to be read from.
    ///
    /// # Return value
    ///
    /// `u16` type.
    pub fn get_16b_reg(&self, reg: &Register16b) -> u16 {
        match reg {
            Register16b::AF => Registers::combine_high_low_bytes(self.a, self.f),
            Register16b::BC => Registers::combine_high_low_bytes(self.b, self.c),
            Register16b::DE => Registers::combine_high_low_bytes(self.d, self.e),
            Register16b::HL => Registers::combine_high_low_bytes(self.h, self.l),
            Register16b::SP => self.sp,
            Register16b::PC => self.pc,
        }
    }

    fn combine_high_low_bytes(high_byte: u8, low_byte: u8) -> u16 {
        let high: u16 = (high_byte as u16) << 8;
        high + low_byte as u16
    }

    fn split_high_low_bytes(value: u16) -> (u8, u8) {
        let high: u8 = (value >> 8) as u8;
        let low: u8 = (value & 0x00FF) as u8;
        (high, low)
    }

    /// Sets desired flag in CPU register F to 1.
    ///
    /// # Arguments
    ///
    /// * `flag` - The flag to be set. One of `Flag::{Z, N, H, C}`.
    pub fn set_flag(&mut self, flag: &Flag) {
        match flag {
            Flag::Z => self.f |= 0b_1000_0000,
            Flag::N => self.f |= 0b_0100_0000,
            Flag::H => self.f |= 0b_0010_0000,
            Flag::C => self.f |= 0b_0001_0000,
        }
    }

    /// Clears desired flag in CPU register F to 0.
    ///
    /// # Arguments
    ///
    /// * `flag` - The flag to be cleared. One of `Flag::{Z, N, H, C}`.
    pub fn clear_flag(&mut self, flag: &Flag) {
        match flag {
            Flag::Z => self.f &= 0b_0111_1111,
            Flag::N => self.f &= 0b_1011_1111,
            Flag::H => self.f &= 0b_1101_1111,
            Flag::C => self.f &= 0b_1110_1111,
        }
    }

    /// Returns the valued stored in one of the CPU flags.
    ///
    /// # Arguments
    ///
    /// * `flag` - The flag to be read from. One of Flag::{Z, N, H, C}.
    ///
    /// # Return value
    ///
    /// * `bool` type where
    /// | return value  | flag state           |
    /// |:-------------:|:--------------------:|
    /// | `true`        | `register.flag == 1` |
    /// | `false `      | `register.flag == 0` |
    pub fn get_flag(&self, flag: &Flag) -> bool {
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

/// unit tests for CPU registers
/// 
/// NOTE: test functions named as `register_{TYPE}_{TEST}...`, where `TEST` refers to the functionality/cases tested for
#[cfg(test)]
mod tests {
    use super::*;

    /// test helper module
    mod common {
        use super::super::*;
        pub fn all_registers_8b() -> Vec<Register8b> {
            vec![
                Register8b::A,
                Register8b::F,
                Register8b::B,
                Register8b::C,
                Register8b::D,
                Register8b::E,
                Register8b::H,
                Register8b::L,
            ]
        }

        pub fn all_registers_16b() -> Vec<Register16b> {
            vec![
                Register16b::AF,
                Register16b::BC,
                Register16b::DE,
                Register16b::HL,
                Register16b::SP,
                Register16b::PC,
            ]
        }

        pub fn all_flags() -> Vec<Flag> {
            vec![
                Flag::Z,
                Flag::N,
                Flag::H,
                Flag::C,
            ]
        }
    }

    #[test]
    fn register_8b_set_get() {
        let registers_8b = common::all_registers_8b();

        let mut registers = Registers::new();

        for reg in &registers_8b {
            registers.set_8b_reg(reg, 0x1)
        }

        for reg in &registers_8b {
            assert_eq!(1u8, registers.get_8b_reg(reg))
        }
    }

    #[test]
    fn register_8b_init_zero_get() {
        let registers_8b = common::all_registers_8b();

        let registers = Registers::new();

        for reg in &registers_8b {
            assert_eq!(0u8, registers.get_8b_reg(reg));
        }
    }

    #[test]
    fn register_16b_init_zero_get() {
        let registers_16b = common::all_registers_16b();

        let registers = Registers::new();

        for reg in &registers_16b {
            assert_eq!(0u16, registers.get_16b_reg(reg));
        }
    }

    #[test]
    fn register_16b_set_get() {
        let registers_16b = common::all_registers_16b();

        let mut registers = Registers::new();

        for reg in &registers_16b {
            registers.set_16b_reg(reg, 0xF0F0u16)
        }

        for reg in &registers_16b {
            assert_eq!(0xF0F0u16, registers.get_16b_reg(reg));
        }
    }

    #[test]
    fn register_16b_set_8b_get() {
        let mut registers = Registers::new();

        let registers_16b = common::all_registers_16b();

        for reg_16b in &registers_16b {
            registers.set_16b_reg(reg_16b, 0xF0F0u16)
        }

        let registers_8b = common::all_registers_8b();
        for reg_8b in &registers_8b {
            assert_eq!(0xF0u8, registers.get_8b_reg(reg_8b));
        }
    }

    #[test]
    fn register_8b_set_16b_get() {
        let mut registers = Registers::new();
        
        let registers_8b = common::all_registers_8b();

        for reg_8b in &registers_8b {
            registers.set_8b_reg(reg_8b, 0xF0u8);
        }

        let registers_16b = common::all_registers_16b();

        for reg_16b in &registers_16b[0..4] { // exclude SP, PC registers
            println!("{}", registers);
            assert_eq!(0xF0F0u16, registers.get_16b_reg(reg_16b));
        }

    }

    #[test]
    fn register_flag_init_zero_get() {
        let flags = common::all_flags();

        let mut registers = Registers::new();

        for flag in &flags {
            assert_eq!(false, registers.get_flag(flag));
        }
    }

    #[test]
    fn register_flag_set_clear() {
        let flags = common::all_flags();

        let mut registers = Registers::new();
        // set all flags and check for true
        for flag in &flags {
            registers.set_flag(flag);
            assert_eq!(true, registers.get_flag(flag));
        }
        // clear all flags and check for false
        for flag in &flags {
            registers.clear_flag(flag);
            assert_eq!(false, registers.get_flag(flag));
        }
    }

}
