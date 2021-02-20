// struct Registers {
//     // 8 bit registers
//     a:  u8,
//     f:  u8,
//     b:  u8,
//     c:  u8,
//     d:  u8,
//     e:  u8,
//     h:  u8,
//     l:  u8,
//     // 16 bit registers
//     sp: u16,
//     pc: u16,
// }

// impl<'a> Registers<'a> {
//     fn new() -> Registers<'a> {
//         Registers {
            
//         }
//     }
// }

// struct CPU<'a> {
//     registers: Registers<'a>,
// }

// impl<'a> CPU<'a> {
//     fn new() -> CPU<'a> {
//         CPU {
//             registers: Registers::new(),
//             a: &0,
//             f: &0,
//             af: &0,
//         }
//     }
// }

// AB
// A <- 16bit

// 5 * 16
// a: u8,
// f: u8,
// b: u8,
// c: u8,

mod register;


fn main() {
    register::Registers::new();

}