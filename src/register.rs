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

    pub fn get_16b_reg(reg: &str) -> u16 {
        match reg {
            "af" => {
                let high: u16 = self.a as u16  << 8
                high + self.f as u16
            },
            "bc" => {
                let high: u16 = self.b as u16 << 8
                high + self.c as u16
            },
            "de" => {
                let high: u16 = self.d as u16 << 8
                high + self.e as u16
            },
            "hl" => {
                let high: u16 = self.d as u16 << 8
                high + self.e as u16
            },
            _   => panic!("Invalid register accessed")
        }
    }


}
