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
    //sp: u16,
    //pc: u16,
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

    pub fn get_16b_reg(&self, reg: &str) -> u16 {
        match reg {
            "af" => {
                let high: u16 = (self.a as u16)  << 8;
                high + self.f as u16
            },
            "bc" => {
                let high: u16 = (self.b as u16) << 8;
                high + self.c as u16
            },
            "de" => {
                let high: u16 = (self.d as u16) << 8;
                high + self.e as u16
            },
            "hl" => {
                let high: u16 = (self.d as u16) << 8;
                high + self.e as u16
            },
            _   => panic!("Invalid register accessed")
        }
    }


}


// unit tests for registers
#[cfg(test)]
mod tests {
    use super::*;

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