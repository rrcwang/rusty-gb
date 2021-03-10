mod cpu;
mod memory;
mod registers;

fn main() {
    let x: u16 = 0x02;
    let y: u16 = 0x03;
    let r = x.wrapping_sub(y);

    println!("0x{:X} - 0x{:X} == 0x{:X}", x, y, r);

    let x = 0;
    if true {
        let x = 1;
    }
    println!("{}", x);
}
