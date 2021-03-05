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
    pub sp: u16,
    pub pc: u16,
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

    /// Sets the value in one of the 8 bit CPU registers
    ///
    /// # Arguments
    ///
    /// * `reg` - The register to be written to.
    /// * `value` - u8 value written to the specified register
    pub fn set_8b_reg(&mut self, reg: Register8b, value: u8) {
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
    pub fn get_8b_reg(&self, reg: Register8b) -> u8 {
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
    pub fn set_16b_reg(&mut self, reg: Register16b, value: u16) {
        let split_high_low_bytes = |value: u16| {
            let high: u8 = (value >> 8) as u8;
            let low: u8 = (value & 0x00FF) as u8;
            (high, low)
        };

        match reg {
            Register16b::AF => {
                let (x, y) = split_high_low_bytes(value);
                self.a = x;
                self.f = y & 0xF0; // mask, lower 4 bits should always be 0
            }
            Register16b::BC => {
                let (x, y) = split_high_low_bytes(value);
                self.b = x;
                self.c = y;
            }
            Register16b::DE => {
                let (x, y) = split_high_low_bytes(value);
                self.d = x;
                self.e = y;
            }
            Register16b::HL => {
                let (x, y) = split_high_low_bytes(value);
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
    pub fn get_16b_reg(&self, reg: Register16b) -> u16 {
        let combine_high_low_bytes = |high_byte: u8, low_byte: u8| -> u16 {
            let high: u16 = (high_byte as u16) << 8;
            high + low_byte as u16
        };

        match reg {
            Register16b::AF => combine_high_low_bytes(self.a, self.f),
            Register16b::BC => combine_high_low_bytes(self.b, self.c),
            Register16b::DE => combine_high_low_bytes(self.d, self.e),
            Register16b::HL => combine_high_low_bytes(self.h, self.l),
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
    /// * `flag` - The flag to be read from. One of Flag::{Z, N, H, C}.
    ///
    /// # Return value
    ///
    /// * `bool` type where
    /// | return value  | flag state           |
    /// |:-------------:|:--------------------:|
    /// | `true`        | `register.flag == 1` |
    /// | `false `      | `register.flag == 0` |
    pub fn get_flag(&self, flag: Flag) -> bool {
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
mod test {
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
            vec![Flag::Z, Flag::N, Flag::H, Flag::C]
        }
    }

    #[test]
    fn register_8b_set_get() {
        let registers_8b = common::all_registers_8b();

        let mut registers = Registers::new();

        for reg in registers_8b {
            registers.set_8b_reg(reg, 0x1)
        }

        let registers_8b = common::all_registers_8b();

        for reg in registers_8b {
            assert_eq!(1u8, registers.get_8b_reg(reg))
        }
    }

    #[test]
    fn register_8b_init_zero_get() {
        let registers_8b = common::all_registers_8b();

        let registers = Registers::new();

        for reg in registers_8b {
            assert_eq!(0u8, registers.get_8b_reg(reg));
        }
    }

    #[test]
    fn register_16b_init_zero_get() {
        let registers_16b = common::all_registers_16b();

        let registers = Registers::new();

        for reg in registers_16b {
            assert_eq!(0u16, registers.get_16b_reg(reg));
        }
    }

    #[test]
    fn register_16b_set_get() {
        let registers_16b = common::all_registers_16b();

        let mut registers = Registers::new();

        for reg in registers_16b {
            registers.set_16b_reg(reg, 0xF0F0u16)
        }

        let registers_16b = common::all_registers_16b();

        for reg in registers_16b {
            assert_eq!(0xF0F0u16, registers.get_16b_reg(reg));
        }
    }

    #[test]
    fn register_16b_set_8b_get() {
        let mut registers = Registers::new();

        let registers_16b = common::all_registers_16b();

        for reg_16b in registers_16b {
            registers.set_16b_reg(reg_16b, 0xF0F0u16)
        }

        let registers_8b = common::all_registers_8b();
        for reg_8b in registers_8b {
            assert_eq!(0xF0u8, registers.get_8b_reg(reg_8b));
        }
    }

    #[test]
    fn register_8b_set_16b_get() {
        let mut registers = Registers::new();

        let registers_8b = common::all_registers_8b();

        for reg_8b in registers_8b {
            registers.set_8b_reg(reg_8b, 0xF0u8);
        }

        for reg_16b in vec![
            Register16b::AF,
            Register16b::BC,
            Register16b::DE,
            Register16b::HL,
        ] {
            // AF, BC, DE, HL
            assert_eq!(0xF0F0u16, registers.get_16b_reg(reg_16b));
        }

        for reg_16b in vec![Register16b::SP, Register16b::PC] {
            // SP, PC
            assert_eq!(0, registers.get_16b_reg(reg_16b));
        }
    }

    #[test]
    fn register_flag_init_zero_get() {
        let flags = common::all_flags();

        let registers = Registers::new();

        for flag in flags {
            assert_eq!(false, registers.get_flag(flag));
        }
    }

    #[test]
    fn register_flag_set_clear() {
        let flags = common::all_flags();

        let mut registers = Registers::new();
        // set all flags and check for true
        for flag in flags {
            registers.set_flag(flag, true);
        }

        let flags = common::all_flags();
        for flag in flags {
            assert_eq!(true, registers.get_flag(flag));
        }

        let flags = common::all_flags();
        // clear all flags and check for false
        for flag in flags {
            registers.set_flag(flag, false);
        }

        let flags = common::all_flags();
        for flag in flags {
            assert_eq!(false, registers.get_flag(flag));
        }
    }

    #[test]
    #[ignore] // time consuming test
    fn benchmark_16b_write() {
        use std::time::Instant;

        let mut regs = Registers::new();

        let n_test: u32 = 100000000;

        let now = Instant::now();

        for _ in 0..n_test {
            regs.set_16b_reg(Register16b::AF, 0xFFFF);
            regs.set_16b_reg(Register16b::AF, 0x0000);
        }

        println!("u16 set time: {} ms", now.elapsed().as_millis());
        // on average, about 4.0s.
        // unsafe, raw pointer access is about 3.6s.
        // maybe change implementation for performance if necessary??
    }
}
