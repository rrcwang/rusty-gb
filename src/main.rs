mod registers;
mod cpu;
mod memory;

use std::mem::size_of_val;

fn main() {
    let z80 = cpu::CPU::new();

}
