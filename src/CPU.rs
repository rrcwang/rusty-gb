pub struct CPU {
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
    SP,
    PC,
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
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

    /// Sets the value in one of the 8 bit registers
    pub fn set_8b_reg(&mut self, reg: Register8b, value: u8) {
        match reg {
            Register8b::A => self.a = value,
            Register8b::F => self.f = value,
            Register8b::B => self.b = value,
            Register8b::C => self.c = value,
            Register8b::D => self.d = value,
            Register8b::E => self.e = value,
            Register8b::H => self.h = value,
            Register8b::L => self.l = value,
        }
    }

    /// Retrieves value stored in one of the 8 bit registers
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

    /// Sets the value in one of the 16 bit registers
    pub fn set_16b_reg(&mut self, reg: Register16b, value: u16) {
        match reg {
            Register16b::AF => {
                let (x, y) = CPU::split_high_low_bytes(value);
                self.a = x;
                self.f = y;
            }
            Register16b::BC => {
                let (x, y) = CPU::split_high_low_bytes(value);
                self.b = x;
                self.c = y;
            }
            Register16b::DE => {
                let (x, y) = CPU::split_high_low_bytes(value);
                self.d = x;
                self.e = y;
            }
            Register16b::HL => {
                let (x, y) = CPU::split_high_low_bytes(value);
                self.h = x;
                self.l = y;
            }
            Register16b::SP => self.sp = value,
            Register16b::PC => self.pc = value,
        }
    }

    pub fn get_16b_reg(&self, reg: Register16b) -> u16 {
        match reg {
            Register16b::AF => CPU::combine_high_low_bytes(self.a, self.f),
            Register16b::BC => CPU::combine_high_low_bytes(self.b, self.c),
            Register16b::DE => CPU::combine_high_low_bytes(self.d, self.e),
            Register16b::HL => CPU::combine_high_low_bytes(self.h, self.l),
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
}

// unit tests for CPU registers
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_set_get_8b() {
        let registers_8b = vec![Register8b::A, Register8b::F, Register8b::B, Register8b::C, Register8b::D, Register8b::E, Register8b::H, Register8b::L];

        let mut cpu = CPU::new();

        for reg in registers_8b {
            cpu.set_8b_reg(reg, 0x1)
        }
        
        let registers_8b = vec![Register8b::A, Register8b::F, Register8b::B, Register8b::C, Register8b::D, Register8b::E, Register8b::H, Register8b::L];

        for reg in registers_8b {
            assert_eq!(1u8, cpu.get_8b_reg(reg))
        }
    }

    #[test]
    fn register_init_zero_8b() {
        let registers_8b = vec![Register8b::A, Register8b::F, Register8b::B, Register8b::C, Register8b::D, Register8b::E, Register8b::H, Register8b::L];

        let cpu = CPU::new();

        for reg in registers_8b {
            assert_eq!(0u8, cpu.get_8b_reg(reg));
        }
    }

    #[test]
    fn register_init_zero_16b() {
        let registers_16b = vec![Register16b::AF, Register16b::BC, Register16b::DE, Register16b::HL, Register16b::SP, Register16b::PC];

        let cpu = CPU::new();

        for reg in registers_16b {
            assert_eq!(0u16, cpu.get_16b_reg(reg));
        }
    }

    #[test]
    fn register_set_get_16b() {
        let registers_16b = vec![Register16b::AF, Register16b::BC, Register16b::DE, Register16b::HL, Register16b::SP, Register16b::PC];

        let mut cpu = CPU::new();

        for reg in registers_16b {
            cpu.set_16b_reg(reg, 0xF0F0u16)
        }
        
        let registers_16b = vec![Register16b::AF, Register16b::BC, Register16b::DE, Register16b::HL, Register16b::SP, Register16b::PC];

        for reg in registers_16b {
            assert_eq!(0xF0F0u16, cpu.get_16b_reg(reg));
        }
    }
}
