
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
    // sp: u16,
    // pc: u16,

}

impl Registers {

    pub fn new() -> Registers{
        Registers {
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
        }
    }

    pub fn set_8b_reg(&mut self, reg: &str, value: u8) {
        match reg {
            "a" => self.a = value,
            "f" => self.f = value,
            "b" => self.b = value,
            "c" => self.c = value,
            "d" => self.d = value,
            "e" => self.e = value,
            "h" => self.h = value,
            "l" => self.l = value,
            _   => panic!("Invalid register accessed"),
        }
    }

    fn split_high_low_bytes(value: u16) -> (u8,u8) {
        let high: u8 = (value >> 8) as u8;
        let low: u8 = (value & 0x00FF) as u8;
        (high,low)
    }

    pub fn set_16b_reg(&mut self, reg: &str, value: u16) {
        match reg {
            "af" => {
                let (x, y) = Registers::split_high_low_bytes(value);
                self.a = x;
                self.f = y;
            },
            "bc" => {
                let (x, y) = Registers::split_high_low_bytes(value);
                self.b = x;
                self.c = y;
            },
            "de" => {
                let (x, y) = Registers::split_high_low_bytes(value);
                self.d = x;
                self.e = y;
            },
            "hl" => {let (x, y) = Registers::split_high_low_bytes(value);
                self.h = x;
                self.l = y;
            },
            _   => panic!("Invalid register accessed")
        }
    }

    pub fn get_8b_reg(&self, reg: &str) -> u8 {
        match reg {
            "a" => self.a,
            "f" => self.f,
            "b" => self.b,
            "c" => self.c,
            "d" => self.d,
            "e" => self.e,
            "h" => self.h,
            "l" => self.l,
            _   => panic!("Invalid register accessed"),
        }
    }

    fn combine_high_low_bytes(high_byte: u8, low_byte: u8) -> u16{
        let high: u16 = (high_byte as u16) << 8;
        high + low_byte as u16
    }

    pub fn get_16b_reg(&self, reg: &str) -> u16 {
        match reg {
            "af" => Registers::combine_high_low_bytes(self.a, self.f),
            "bc" => Registers::combine_high_low_bytes(self.b, self.c),
            "de" => Registers::combine_high_low_bytes(self.d, self.e),
            "hl" => Registers::combine_high_low_bytes(self.h, self.l),
            _   => panic!("Invalid register accessed")
        }
    }

}


// unit tests for registers
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_set_get_16b() {
        let registers_16b_str = vec!["af", "bc", "de", "hl"];

        let mut registers = Registers::new();

        for reg in &registers_16b_str {
            registers.set_16b_reg(reg, 0xFFFF)
        }

        for reg in &registers_16b_str {
            assert_eq!(0xFFFFu16, registers.get_16b_reg(reg));

        }

    }

    #[test]
    fn register_set_get_8b() {
        let registers_8b_str = vec!["a", "f", "b", "c", "d", "e", "h", "l"];

        let mut registers = Registers::new();

        for reg in &registers_8b_str {
            registers.set_8b_reg(reg, 0x1)
        }

        for reg in &registers_8b_str {
            assert_eq!(1u8, registers.get_8b_reg(reg))
        }

    }


    #[test]
    fn register_init_zero_8b() {
        let registers_8b_str = vec!["a", "f", "b", "c", "d", "e", "h", "l"];

        let registers = Registers::new();

        for reg in registers_8b_str {
            assert_eq!(0u8, registers.get_8b_reg(reg));
        }
    }

    #[test]
    fn register_init_zero_16b() {
        let registers_16b_str = vec!["af", "bc", "de", "hl"];

        let registers = Registers::new();

        for reg in registers_16b_str {
            assert_eq!(0u16, registers.get_16b_reg(reg));
        }
    }

    #[test]
    #[should_panic]
    fn register_invalid_8b() {
        let registers = Registers::new();

        registers.get_8b_reg("TEST");
    }

    #[test]
    #[should_panic]
    fn register_invalid_16b() {
        let registers = Registers::new();

        registers.get_16b_reg("TEST");
    }
}