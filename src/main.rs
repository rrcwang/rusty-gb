mod cpu;
mod memory;
mod registers;
mod utils;

fn main() {
    let mut cpu = cpu::Cpu::new();
    cpu.fetch_and_execute();
}
