mod registers;

fn main() {
    let z80 = registers::Registers::new();
    let val_8 = z80.get_8b_reg(&registers::Register8b::A);
    let val_16 = z80.get_16b_reg(&registers::Register16b::AF);

    println!("{}", val_8);
}
