use criterion::{black_box, Criterion};
use rusty_gb::cpu::registers::{Register16b, Register8b, Registers};

fn reg_16b_write(regs: &mut Registers) {
    regs.set_r16(Register16b::AF, 0xFFFF);
    regs.set_r16(Register16b::AF, 0x0000);

    // on average, about 4.0s.
    // unsafe, raw pointer access is about 3.6s.
    // maybe change implementation for performance if necessary??
}

pub fn criterion_reg_16b_write(c: &mut Criterion) {
    let mut regs = Registers::new();

    c.bench_function("reg 16b write", |b| {
        b.iter(|| reg_16b_write(black_box(&mut regs)))
    });
}
