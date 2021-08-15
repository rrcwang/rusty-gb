use criterion::{criterion_group, criterion_main};

mod registers;

criterion_group!(benches, registers::criterion_reg_16b_write);
criterion_main!(benches);
