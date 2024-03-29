use super::*;

// test setup functions
mod common {
    use super::super::*;

    pub const REGISTERS_8B_NO_F: &[Register8b] = &[
        Register8b::B,
        Register8b::C,
        Register8b::D,
        Register8b::E,
        Register8b::H,
        Register8b::L,
        Register8b::A,
    ];

    pub const REGISTERS_16B_NO_AF: &[Register16b] = &[
        Register16b::BC,
        Register16b::DE,
        Register16b::HL,
        Register16b::SP,
    ];

    /// Tests that the CPU registers are set correctly
    ///
    /// # Arguments
    ///
    /// * `cpu`
    /// * `values` - Vector specifying values of flags in order of Flag::{Z, N, H, C}
    ///
    /// # Return value
    ///
    /// * `Result::Err(s)` - where `s` is a string describing which flag failed. If multiple failed
    ///                         then only returns the first.
    pub fn assert_flags(cpu: &Cpu, values: &[bool], flags: &[Flag]) -> Result<(), Flag> {
        for (flag_val, flag) in values.iter().zip(flags.iter()) {
            if *flag_val != cpu.registers.flag_value(*flag) {
                return Err(*flag);
            }
        }

        Ok(())
    }

    /// Assert flags for binary operation with bytes (u8)
    ///
    /// # Arguments
    ///
    /// * `cpu`
    /// * `values` - Vector specifying values of flags in order of Flag::{Z, N, H, C}
    /// * `operands` - Tuple of two `u8`s of the binary operation being tested for
    pub fn assert_flags_binop(cpu: &Cpu, values: &[bool; 4], operands: (&u8, &u8)) {
        let flags = &[Flag::Z, Flag::N, Flag::H, Flag::C];
        let result = assert_flags(&cpu, values, flags);

        if let Err(f) = result {
            panic!(
                "Flag assertion failed for flag {:?} with operands (0x{:X}, 0x{:X})",
                f, operands.0, operands.1
            );
        }
    }

    pub fn assert_flags_unaop(cpu: &Cpu, values: &[bool; 4], operand: u8) {
        let flags = &[Flag::Z, Flag::N, Flag::H, Flag::C];
        let result = assert_flags(&cpu, values, flags);

        if let Err(f) = result {
            panic!(
                "Flag assertion failed for flag {:?} with operand {:X}",
                f, operand
            );
        }
    }
}

//////////////////////////////////////////
// Arithmetic, flag tests
//////////////////////////////////////////

#[test]
fn cpu_alu_add_bytes_value() {
    let mut cpu = Cpu::new();

    let test_cases = &[
        (0, 0),
        (1, 1),
        (0xFF, 0xFF),
        (0xFF, 0x01),
        (0x01, 0xFF),
        (0xFE, 0x01),
        (0x01, 0xFE),
        (5, 7),
        (12, 35),
    ];

    for (a, b) in test_cases {
        let result = cpu.alu_add_bytes(*a, *b, false);
        assert_eq!(a.wrapping_add(*b), result);
    }
}

#[test]
fn cpu_alu_add_bytes_flag_z() {
    let mut cpu = Cpu::new();

    let test_cases = &[
        // a    b       Z     N      H      C
        ((0x00, 0x00), [true, false, false, false]),
        ((0xFF, 0x01), [true, false, true, true]),
        ((0xFE, 0x02), [true, false, true, true]),
        ((0x02, 0xFE), [true, false, true, true]),
    ];

    for ((a, b), flags) in test_cases {
        let result: u8 = cpu.alu_add_bytes(*a, *b, false);
        assert_eq!(0, result);

        common::assert_flags_binop(&cpu, &flags, (a, b));
    }
}

#[test]
fn cpu_alu_add_bytes_flag_h() {
    let mut cpu = Cpu::new();

    let test_cases = &[
        // a    b       Z     N      H      C
        ((0x00, 0x01), [false, false, false, false]), // 1
        ((0x08, 0x07), [false, false, false, false]), // 15
        ((0x0F, 0x01), [false, false, true, false]),  // 16
        ((0x0E, 0x02), [false, false, true, false]),  // 16
        ((0x08, 0x08), [false, false, true, false]),  // 16
        ((0x0F, 0x02), [false, false, true, false]),  // 17
        ((0x0E, 0x04), [false, false, true, false]),  // 18
        ((0x0F, 0x0F), [false, false, true, false]),  // 30
        // empty lower bits
        ((0xF0, 0xF0), [false, false, false, true]),
        ((0xF0, 0x20), [false, false, false, true]),
        ((0xF0, 0x01), [false, false, false, false]),
        ((0x50, 0x51), [false, false, false, false]),
    ];

    for ((a, b), flags) in test_cases {
        let result: u8 = cpu.alu_add_bytes(*a, *b, false);
        assert_eq!(a.wrapping_add(*b), result); // unnecessary?

        common::assert_flags_binop(&cpu, flags, (a, b));
    }
}

#[test]
fn cpu_alu_add_bytes_flag_c() {
    let mut cpu = Cpu::new();

    let test_cases = &[
        // a    b       Z     N      H      C
        ((0x00, 0x00), [true, false, false, false]),
        ((0x00, 0x01), [false, false, false, false]),
        ((0xFF, 0x01), [true, false, true, true]),
        ((0xFE, 0x02), [true, false, true, true]),
        ((0xF0, 0x20), [false, false, false, true]),
    ];

    for ((a, b), flags) in test_cases {
        let result: u8 = cpu.alu_add_bytes(*a, *b, false);
        assert_eq!(a.wrapping_add(*b), result);

        common::assert_flags_binop(&cpu, flags, (a, b));
    }
}

#[test]
fn cpu_alu_add_bytes_with_carry() {
    let mut cpu = Cpu::new();
    let test_cases = &[
        // a    b       Z     N      H      C
        ((0x00, 0x00), [false, false, false, false]),
    ];

    for ((a, b), flags) in test_cases {
        cpu.registers.set_flag(Flag::C, true);
        let result = cpu.alu_add_bytes(*a, *b, true);
        assert_eq!(a.wrapping_add(*b).wrapping_add(1), result);

        common::assert_flags_binop(&cpu, flags, (a, b));
    }
}

#[test]
fn cpu_alu_sub_bytes_value() {
    let mut cpu = Cpu::new();

    let test_cases = &[
        (0x00, 0x00),
        (0x01, 0x00),
        (0xFF, 0x32),
        (0x01, 0x32),
        (0x32, 0x32),
    ];

    for (a, b) in test_cases {
        let result = cpu.alu_sub_bytes(*a, *b, false);
        assert_eq!(a.wrapping_sub(*b), result);
    }
}

#[test]
fn cpu_alu_sub_bytes_flag_z() {
    let mut cpu = Cpu::new();

    // NOTE: Let x - y == 0 => x == y.
    //  shouldn't be possible to underflow into 0 if both are u8?
    //  similar reasonining for why H should never be set if x == y.
    let test_cases = &[
        // a    b       Z     N      H      C
        ((0x00, 0x00), [true, true, false, false]),
        ((0x01, 0x01), [true, true, false, false]),
        ((0x1F, 0x1F), [true, true, false, false]),
        ((0xFF, 0xFF), [true, true, false, false]),
    ];

    for ((a, b), flags) in test_cases {
        let result: u8 = cpu.alu_sub_bytes(*a, *b, false);
        assert_eq!(a.wrapping_sub(*b), result);

        common::assert_flags_binop(&cpu, flags, (a, b));
    }
}

#[test]
fn cpu_alu_sub_bytes_flag_h() {
    let mut cpu = Cpu::new();

    let test_cases = &[
        // a    b       Z     N      H      C
        ((0x00, 0x00), [true, true, false, false]),
        ((0x00, 0x01), [false, true, true, true]),
        ((0x03, 0x04), [false, true, true, true]),
        ((0x07, 0x08), [false, true, true, true]),
        ((0x08, 0x09), [false, true, true, true]),
        ((0x0F, 0x10), [false, true, false, true]), //?????
        ((0x10, 0x02), [false, true, true, false]),
    ];

    for ((a, b), flags) in test_cases {
        let result: u8 = cpu.alu_sub_bytes(*a, *b, false);
        assert_eq!(a.wrapping_sub(*b), result);

        common::assert_flags_binop(&cpu, flags, (a, b));
    }
}

#[test]
fn cpu_alu_sub_bytes_flag_c() {
    let mut cpu = Cpu::new();

    let test_cases = &[
        // a    b       Z     N      H      C
        ((0xFE, 0xFF), [false, true, true, true]),
        ((0x00, 0x01), [false, true, true, true]),
    ];

    for ((a, b), flags) in test_cases {
        let result: u8 = cpu.alu_sub_bytes(*a, *b, false);
        assert_eq!(a.wrapping_sub(*b), result);

        common::assert_flags_binop(&cpu, flags, (a, b));
    }
}

#[test]
fn cpu_alu_sub_bytes_with_carry() {
    let mut cpu = Cpu::new();

    let test_cases = &[
        //  a   b
        (0x01, 0x00),
        (0x01, 0x01),
        (0x10, 0x02),
        (0x10, 0x1F),
    ];

    for (a, b) in test_cases {
        cpu.registers.set_flag(Flag::C, true);
        let result: u8 = cpu.alu_sub_bytes(*a, *b, true);

        assert_eq!(a.wrapping_sub(*b).wrapping_sub(1), result);
    }
}

#[test]
fn cpu_alu_inc_byte() {
    let mut cpu = Cpu::new();

    let test_cases = &[
        //      Z      N      H
        (0xFE, [false, false, false]),
    ];

    let flags = &[Flag::Z, Flag::N, Flag::H];
    for (a, flag_vals) in test_cases {
        let value = cpu.alu_inc_byte(*a);

        assert_eq!(value, a.wrapping_add(1));

        // check flag assertion
        let result = common::assert_flags(&cpu, flag_vals, flags);
        if let Err(f) = result {
            panic!("Flag assertion failed for {:?} with value {:X}.", f, a);
        }
    }
}

#[test]
fn cpu_alu_dec_byte() {
    let mut cpu = Cpu::new();

    let test_cases = &[
        //        Z      N     H
        (0xFF, [false, true, false]),
        (0x00, [false, true, true]),
        (0x9F, [false, true, false]),
        (0x13, [false, true, false]),
        (0x01, [true, true, false]),
        (0x0F, [false, true, false]),
        (0x10, [false, true, true]),
    ];

    let flags = &[Flag::Z, Flag::N, Flag::H];
    for (a, flag_vals) in test_cases {
        let value = cpu.alu_dec_byte(*a);

        assert_eq!(value, a.wrapping_sub(1));

        // check flag assertion
        let result = common::assert_flags(&cpu, flag_vals, flags);
        if let Err(f) = result {
            panic!("Flag assertion failed for {:?} with value {:X}.", f, a);
        }
    }
}

#[test]
fn cpu_alu_add_words() {
    let mut cpu = Cpu::new();

    let test_cases = &[
        //                N      H      C
        (0x0000, 0x0000, [false, false, false]),
        (0x1523, 0x2333, [false, false, false]),
        (0x0100, 0x0F00, [false, true, false]),
        (0x1000, 0xF000, [false, false, true]),
        (0x0100, 0xFF00, [false, true, true]),
        (0x0101, 0x1010, [false, false, false]),
        (0x0FFF, 0x0001, [false, true, false]),
    ];

    let flags = &[Flag::N, Flag::H, Flag::C];
    for (x, y, flag_vals) in test_cases {
        let value = cpu.alu_add_words(*x, *y);
        assert_eq!(value, x.wrapping_add(*y));

        let result = common::assert_flags(&cpu, flag_vals, flags);
        if let Err(f) = result {
            panic!(
                "Flag assertion failed for {:?} with values {:?}.",
                f,
                (x, y)
            );
        }
    }
}

#[test]
fn cpu_alu_and_a() {
    let mut cpu = Cpu::new();

    let test_cases = &[
        // a   y     Z     N     H     C
        (0x00, 0x00, [true, false, true, false]),
        (0x00, 0xFF, [true, false, true, false]),
        (0xFF, 0x00, [true, false, true, false]),
        (0x1F, 0x00, [true, false, true, false]),
        (0x11, 0x22, [true, false, true, false]), // TODO more test cases needed
    ];

    for (a, y, flags) in test_cases {
        cpu.registers.set_r8(Register8b::A, *a);
        cpu.alu_and_a(*y);

        assert_eq!(a & y, cpu.registers.get_r8(Register8b::A));
        common::assert_flags_binop(&cpu, flags, (a, y));
    }
}

#[test]
fn cpu_alu_xor_a() {
    let mut cpu = Cpu::new();

    let test_cases = &[
        // a   y     Z     N     H     C
        (0x00, 0x00, [true, false, false, false]),
        (0xFF, 0xFF, [true, false, false, false]),
        (0xF0, 0x0F, [false, false, false, false]),
        (0xAA, 0x55, [false, false, false, false]),
    ];

    for (a, y, flags) in test_cases {
        cpu.registers.set_r8(Register8b::A, *a);
        cpu.alu_xor_a(*y);

        assert_eq!(a ^ y, cpu.registers.get_r8(Register8b::A));
        common::assert_flags_binop(&cpu, flags, (a, y));
    }
}

#[test]
fn cpu_alu_or_a() {
    let mut cpu = Cpu::new();

    let test_cases = &[
        // a   y     Z     N     H     C
        (0x00, 0x00, [true, false, false, false]), // TODO more test cases needed
        (0xFF, 0xFF, [false, false, false, false]),
        (0xF0, 0x0F, [false, false, false, false]),
        (0xAA, 0x55, [false, false, false, false]),
        (0x00, 0xF0, [false, false, false, false]),
    ];

    for (a, y, flags) in test_cases {
        cpu.registers.set_r8(Register8b::A, *a);
        cpu.alu_or_a(*y);

        assert_eq!(a | y, cpu.registers.get_r8(Register8b::A));
        common::assert_flags_binop(&cpu, flags, (a, y));
    }
}

#[test]
fn cpu_alu_cp_a() {
    // flag tests should be covered by cpu_alu_sub_bytes
    let mut cpu = Cpu::new();

    let test_cases = &[
        // a   y     Z     N     H     C
        (0x00, 0x00, [true, true, false, false]),
    ];

    for (a, y, flags) in test_cases {
        cpu.registers.set_r8(Register8b::A, *a);
        cpu.alu_cp_a(*y);

        assert_eq!(a, &cpu.registers.get_r8(Register8b::A));
        common::assert_flags_binop(&cpu, flags, (a, y));
    }
}

#[test]
fn cpu_fetch_byte() {
    let mut cpu = Cpu::new();

    let test_rom: Vec<u8> = vec![0x00, 0x50, 0x12, 0x55];
    cpu.mmu.load_rom(test_rom.clone());

    cpu.registers.pc = 0;
    let mut test_pc: u16 = 0;
    for v in test_rom {
        assert_eq!(v, cpu.fetch_byte());
        test_pc += 1;
        assert_eq!(test_pc, cpu.registers.pc);
    }
}

#[test]
fn cpu_fetch_word() {
    let mut cpu = Cpu::new();

    let test_vals: Vec<u16> = vec![0x0000, 0x5012, 0x12AE, 0x5EE5, 0xFFFF, 0x1283];

    let mut test_rom: Vec<u8> = Vec::new();
    for v in &test_vals {
        let (high, low) = word_to_bytes(*v);
        test_rom.push(low);
        test_rom.push(high);
    }
    cpu.mmu.load_rom(test_rom);

    cpu.registers.pc = 0;
    let mut test_pc: u16 = 0;
    for v in test_vals {
        assert_eq!(v, cpu.fetch_word());
        test_pc += 2;
        assert_eq!(test_pc, cpu.registers.pc);
    }
}

//////////////////////////////////////////
// Instruction tests
//////////////////////////////////////////
// Should verify that the correct registers are set.
// The flags should be checked as well if not covered by
// the ALU unit tests above.

#[test]
/// Generates all possible permutations of LD R8, R8 instructions and stores them into a mocked
/// test rom.
fn cpu_instr_ld_r8_r8() {
    let mut cpu = Cpu::new();

    let mut test_cases: Vec<(u8, Register8b, Register8b)> = Vec::new();
    let mut op_code: u8 = 0x40;
    let mut ops: Vec<u8> = Vec::new();
    for reg_to in common::REGISTERS_8B_NO_F {
        for reg_from in common::REGISTERS_8B_NO_F {
            test_cases.push((op_code, *reg_to, *reg_from));
            ops.push(op_code);

            op_code += 1;

            if op_code % 8 == 6 {
                op_code += 1;
            } else if op_code == 0x70 {
                op_code += 8;
            }
        }
    }

    let mut test_pc = 0x00;
    cpu.registers.pc = 0x00;
    cpu.mmu.load_rom(ops); // NOTE: unstable API for rom loading! subject to change

    let test_value: u8 = 0x0F;
    for (op, reg_to, reg_from) in test_cases {
        cpu.registers.set_r8(reg_from, test_value);
        cpu.fetch_and_execute();
        assert_eq!(
            test_value,
            cpu.registers.get_r8(reg_to),
            "OP failed: 0x{:X}. From {:?} to {:?}.\n Registers dump: {}",
            op,
            reg_from,
            reg_to,
            cpu.registers
        );
        // check that pc has been correctly incremented
        test_pc += 1;
        assert_eq!(test_pc, cpu.registers.pc);

        // clear registers
        cpu.registers.set_r8(reg_to, 0);
        cpu.registers.set_r8(reg_from, 0);
    }
}

#[test]
fn cpu_instr_ld_r16_d16() {
    let test_cases: Vec<u16> = vec![0x0032, 0x0000, 0x0001, 0xFFFF, 0xFF32, 0x5050, 0x2312];

    let instrs: Vec<u8> = vec![0x01, 0x11, 0x21, 0x31];
    let instrs = instrs.into_iter().zip(common::REGISTERS_16B_NO_AF);

    let mut cpu = Cpu::new();
    for x in test_cases {
        let bytes = word_to_bytes(x);
        for (op, reg) in instrs.clone() {
            // mocked rom test
            let test_rom = vec![op, bytes.1, bytes.0];
            cpu.mmu.load_rom(test_rom);
            cpu.registers.pc = 0;

            cpu.fetch_and_execute();

            assert_eq!(3, cpu.registers.pc);
            let reg_val = cpu.registers.get_r16(*reg);
            assert_eq!(x, reg_val, "0x{:X}, 0x{:X}", x, reg_val);
        }
    }
}

#[test]
fn cpu_instr_ld_r8_d8() {
    let test_cases: Vec<u8> = vec![0x23, 0x55, 0x00, 0x53, 0xFE, 0xAD, 0xFF];
    let instrs: Vec<(u8, Register8b)> = vec![
        (0x06, Register8b::B),
        (0x16, Register8b::D),
        (0x26, Register8b::H),
    ];

    let mut cpu = Cpu::new();
    for value in test_cases {
        for (op, reg) in instrs.clone() {
            let test_rom: Vec<u8> = vec![op, value];
            cpu.registers.pc = 0; // mocked program counter
            cpu.mmu.load_rom(test_rom);

            let cycles = cpu.fetch_and_execute();

            assert_eq!(2, cpu.registers.pc);
            assert_eq!(value, cpu.registers.get_r8(reg));
            assert_eq!(8, cycles);
        }
    }
}

#[test]
fn cpu_instr_ld_hl_ptr_d8() {
    let test_cases: Vec<(u8, u16)> = vec![
        //  val     wram addr
        (0x23, 0xCFFF),
        (0x55, 0xDADE),
        (0x00, 0xC039),
        (0x53, 0xC000),
        (0xFE, 0xCAA9),
        (0xAD, 0xC0A0),
        (0xFF, 0xC00E),
    ];

    let mut cpu = Cpu::new();
    for (value, address) in test_cases {
        cpu.registers.set_r16(Register16b::HL, address);
        let test_rom: Vec<u8> = vec![0x36, value];
        cpu.registers.pc = 0; // mocked program counter
        cpu.mmu.load_rom(test_rom);

        let cycles = cpu.fetch_and_execute();

        assert_eq!(2, cpu.registers.pc);
        assert_eq!(value, cpu.mmu.read_byte(address));
        assert_eq!(12, cycles);
    }
}

#[test]
fn cpu_instr_ld_bc_ptr_a() {
    let test_cases: Vec<(u8, u16)> = vec![
        //  val    addr in 0xC000..=0xDFFF, WRAM range
        (0x00, 0xC000),
        (0x23, 0xC000),
        (0x01, 0xC001),
        (0xFE, 0xD98F),
        (0xEE, 0xCDCD),
        (0x50, 0xD98E),
    ];

    let mut cpu = Cpu::new();
    cpu.registers.pc = 0; // test program counter, shouldn't change
    for (value, address) in test_cases {
        cpu.registers.set_r16(Register16b::BC, address);
        cpu.registers.set_r8(Register8b::A, value);
        let cycles = cpu.execute_instr(0x02);

        assert_eq!(8, cycles);
        assert_eq!(value, cpu.mmu.read_byte(address));
        assert_eq!(cpu.registers.pc, 0);
    }
}

#[test]
fn cpu_instr_ld_de_ptr_a() {
    let test_cases: Vec<(u8, u16)> = vec![
        //  val    addr in 0xC000..=0xDFFF, WRAM range
        (0x00, 0xC000),
        (0x23, 0xC000),
        (0x01, 0xC001),
        (0xFE, 0xD98F),
        (0xEE, 0xCDCD),
        (0x50, 0xD98E),
    ];

    let mut cpu = Cpu::new();
    cpu.registers.pc = 0; // test program counter, shouldn't change
    for (value, address) in test_cases {
        cpu.registers.set_r16(Register16b::DE, address);
        cpu.registers.set_r8(Register8b::A, value);
        let cycles = cpu.execute_instr(0x12);

        assert_eq!(8, cycles);
        assert_eq!(value, cpu.mmu.read_byte(address));
        assert_eq!(cpu.registers.pc, 0);
    }
}

#[test]
fn cpu_instr_ld_hla_ptr_a() {
    let test_cases: Vec<(u8, u16)> = vec![
        //  val    addr in 0xC000..=0xDFFF, WRAM range
        (0x00, 0xC000),
        (0x23, 0xC000),
        (0x01, 0xC001),
        (0xFE, 0xD98F),
        (0xEE, 0xCDCD),
        (0x50, 0xD98E),
    ];

    let mut cpu = Cpu::new();
    cpu.registers.pc = 0; // test program counter, shouldn't change
    for (value, address) in test_cases {
        cpu.registers.set_r16(Register16b::HL, address);
        cpu.registers.set_r8(Register8b::A, value);
        let cycles = cpu.execute_instr(0x22);

        assert_eq!(8, cycles);
        assert_eq!(
            address.wrapping_add(1),
            cpu.registers.get_r16(Register16b::HL)
        );
        assert_eq!(value, cpu.mmu.read_byte(address));
        assert_eq!(cpu.registers.pc, 0);
    }
}

#[test]
fn cpu_instr_ld_hld_ptr_a() {
    let test_cases: Vec<(u8, u16)> = vec![
        //  val    addr in 0xC000..=0xDFFF, WRAM range
        (0x00, 0xC000),
        (0x23, 0xC000),
        (0x01, 0xC001),
        (0xFE, 0xD98F),
        (0xEE, 0xCDCD),
        (0x50, 0xD98E),
    ];

    let mut cpu = Cpu::new();
    cpu.registers.pc = 0; // test program counter, shouldn't change
    for (value, address) in test_cases {
        cpu.registers.set_r16(Register16b::HL, address);
        cpu.registers.set_r8(Register8b::A, value);
        let cycles = cpu.execute_instr(0x32);

        assert_eq!(8, cycles);
        assert_eq!(
            address.wrapping_sub(1),
            cpu.registers.get_r16(Register16b::HL)
        );
        assert_eq!(value, cpu.mmu.read_byte(address));
        assert_eq!(0, cpu.registers.pc);
    }
}

#[test]
fn cpu_instr_ld_d16_ptr_sp() {
    let test_cases: Vec<(u16, u16)> = vec![
        //  addr in 0xC000..=0xDFFF, WRAM range, sp
        (0xC000, 0x8213),
        (0xC000, 0x8213),
        (0xC001, 0x8213),
        (0xD98F, 0x8213),
        (0xCDCD, 0x8213),
        (0xD98E, 0x8213),
    ];

    let mut cpu = Cpu::new();
    cpu.registers.pc = 0; // test program counter, shouldn't change
    for (address, stack_pointer) in test_cases {
        cpu.registers.set_r16(Register16b::HL, address);
        let (high, low) = word_to_bytes(address);
        let test_rom: Vec<u8> = vec![0x08, low, high];
        cpu.mmu.load_rom(test_rom);

        cpu.registers.pc = 0; // mocked program counter
        cpu.registers.sp = stack_pointer;

        let cycles = cpu.fetch_and_execute();

        assert_eq!(20, cycles);
        assert_eq!(3, cpu.registers.pc);
        assert_eq!(stack_pointer, cpu.mmu.read_word(address));
    }
}

#[test]
fn cpu_instr_inc_byte() {
    let test_cases: Vec<u8> = vec![0x00, 0x01, 0x10, 0x0F, 0xFF];

    let mut cpu = Cpu::new();

    let instrs: Vec<u8> = vec![
        0x04, // B
        0x0C, // C
        0x14, // D
        0x1C, // E
        0x24, // H
        0x2C, // L
        0x3C, // A
    ];

    let instrs = instrs.into_iter().zip(common::REGISTERS_8B_NO_F);

    for test_value in test_cases {
        for (op, reg) in instrs.clone() {
            cpu.registers.set_r8(*reg, test_value);
            cpu.execute_instr(op);

            assert_eq!(cpu.registers.get_r8(*reg), test_value.wrapping_add(1));
        }
    }
}

#[test]
fn cpu_instr_inc_hl_ptr() {
    let test_cases: Vec<(u8, u16)> = vec![
        //  val    addr in 0xC000..=0xDFFF, WRAM range
        (0x00, 0xC000),
        (0x23, 0xC000),
        (0x01, 0xC001),
        (0xFE, 0xD98F),
        (0xEE, 0xCDCD),
        (0x50, 0xD98E),
        (0xFF, 0xC949),
    ];

    let mut cpu = Cpu::new();
    cpu.registers.pc = 0; // test program counter, shouldn't change
    for (value, address) in test_cases {
        cpu.mmu.write_byte(address, value);
        cpu.registers.set_r16(Register16b::HL, address);
        let cycles = cpu.execute_instr(0x34);

        assert_eq!(12, cycles);
        assert_eq!(value.wrapping_add(1), cpu.mmu.read_byte(address));
        assert_eq!(cpu.registers.pc, 0);
    }
}

#[test]
fn cpu_instr_dec_hl_ptr() {
    let test_cases: Vec<(u8, u16)> = vec![
        //  val    addr in 0xC000..=0xDFFF, WRAM range
        (0x00, 0xC000),
        (0x23, 0xC000),
        (0x01, 0xC001),
        (0xFE, 0xD98F),
        (0xEE, 0xCDCD),
        (0x50, 0xD98E),
        (0xFF, 0xC949),
    ];

    let mut cpu = Cpu::new();
    cpu.registers.pc = 0; // test program counter, shouldn't change
    for (value, address) in test_cases {
        cpu.mmu.write_byte(address, value);
        cpu.registers.set_r16(Register16b::HL, address);
        let cycles = cpu.execute_instr(0x35);

        assert_eq!(12, cycles);
        assert_eq!(value.wrapping_sub(1), cpu.mmu.read_byte(address));
        assert_eq!(cpu.registers.pc, 0);
    }
}

#[test]
fn cpu_instr_inc_word() {
    let test_cases: Vec<u16> = vec![0x0032, 0x0000, 0x0001, 0xFFFF, 0xFF32, 0x5050, 0x2312];

    let instrs: Vec<u8> = vec![0x03, 0x13, 0x23, 0x33];
    let instrs = instrs.into_iter().zip(common::REGISTERS_16B_NO_AF);

    let mut cpu = Cpu::new();
    for x in test_cases {
        for (op, reg) in instrs.clone() {
            cpu.registers.set_r16(*reg, x);
            cpu.execute_instr(op);

            assert_eq!(
                x.wrapping_add(1),
                cpu.registers.get_r16(*reg),
                "Op: 0x{:X}. {:?}, 0x{:X}",
                op,
                reg,
                x
            );
            cpu.registers.set_r16(*reg, 0x0000);
        }
    }
}

#[test]
fn cpu_instr_dec_byte() {
    let test_cases: Vec<u8> = vec![0x00, 0x01, 0x10, 0x0F, 0xFF];

    let instrs: Vec<u8> = vec![
        0x05, // B
        0x0D, // C
        0x15, // D
        0x1D, // E
        0x25, // H
        0x2D, // L
        0x3D, // A
    ];
    let instrs = instrs.into_iter().zip(common::REGISTERS_8B_NO_F);

    let mut cpu = Cpu::new();
    for test_value in test_cases {
        for (op, reg) in instrs.clone() {
            cpu.registers.set_r8(*reg, test_value);
            cpu.execute_instr(op);

            assert_eq!(cpu.registers.get_r8(*reg), test_value.wrapping_sub(1));
        }
    }
}

#[test]
fn cpu_instr_dec_word() {
    let test_cases: Vec<u16> = vec![0x0032, 0x0000, 0x0001, 0xFFFF, 0xFF32, 0x5050, 0x2312];

    let instrs: Vec<u8> = vec![0x0B, 0x1B, 0x2B, 0x3B];
    let instrs = instrs.into_iter().zip(common::REGISTERS_16B_NO_AF);

    let mut cpu = Cpu::new();
    for x in test_cases {
        for (op, reg) in instrs.clone() {
            cpu.registers.set_r16(*reg, x);
            cpu.execute_instr(op);

            assert_eq!(
                x.wrapping_sub(1),
                cpu.registers.get_r16(*reg),
                "Op: 0x{:X}. {:?}, 0x{:X}",
                op,
                reg,
                x
            );
            cpu.registers.set_r16(*reg, 0x0000);
        }
    }
}

#[test]
fn cpu_instr_add_bytes() {
    let test_cases = &[
        // A    r8
        (0x00, 0x00),
        (0x23, 0x33),
        (0x54, 0xFF),
        (0x01, 0x15),
        (0xFF, 0xFF),
        (0x01, 0x01),
        (0x55, 0x27),
    ];

    let instrs: Vec<u8> = vec![0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x87];
    let instrs = instrs.into_iter().zip(common::REGISTERS_8B_NO_F);

    let mut cpu = Cpu::new();
    for (x, y) in test_cases {
        for (op, reg) in instrs.clone() {
            cpu.registers.set_r8(Register8b::A, *x);
            cpu.registers.set_r8(*reg, *y);

            cpu.execute_instr(op);

            match reg {
                Register8b::A => {
                    assert_eq!(
                        y.wrapping_add(*y),
                        cpu.registers.get_r8(Register8b::A),
                        "Op: 0x{:X}. Assertion failed for ADD A, A for operand 0x{:X})",
                        op,
                        y
                    );
                }
                _ => {
                    assert_eq!(
                        x.wrapping_add(*y),
                        cpu.registers.get_r8(Register8b::A),
                        "Op: 0x{:X}. Assertion failed for ADD A, {:?} for operands (0x{:X}, 0x{:X})",
                        op,
                        reg,
                        x,
                        y
                    );
                }
            }
        }
    }
}

#[test]
fn cpu_instr_adc_bytes() {
    let test_cases = &[
        // A    r8,  carry
        (0x00, 0x00),
        (0x23, 0x33),
        (0x54, 0xFF),
        (0x01, 0x15),
        (0xFF, 0xFF),
        (0x01, 0x01),
        (0x55, 0x27),
    ];

    let instrs: Vec<u8> = vec![0x88, 0x89, 0x8A, 0x8B, 0x8C, 0x8D, 0x8F];
    let instrs = instrs.into_iter().zip(common::REGISTERS_8B_NO_F);

    let mut cpu = Cpu::new();
    for (x, y) in test_cases {
        for (op, reg) in instrs.clone() {
            cpu.registers.set_r8(Register8b::A, *x);
            cpu.registers.set_r8(*reg, *y);
            cpu.registers.set_flag(Flag::C, true);

            cpu.execute_instr(op);

            match reg {
                Register8b::A => {
                    assert_eq!(
                        y.wrapping_add(*y).wrapping_add(1),
                        cpu.registers.get_r8(Register8b::A),
                        "Op: 0x{:X}. Assertion failed for ADC A, A for operand 0x{:X})",
                        op,
                        y
                    );
                }
                _ => {
                    assert_eq!(
                        x.wrapping_add(*y).wrapping_add(1),
                        cpu.registers.get_r8(Register8b::A),
                        "Op: 0x{:X}. Assertion failed for ADC A, {:?} for operands (0x{:X}, 0x{:X})",
                        op,
                        reg,
                        x,
                        y
                    );
                }
            }
        }
    }
}

#[test]
fn cpu_instr_add_words() {
    let test_cases = &[
        // HL    r16
        (0x0000, 0x0000),
        (0x1523, 0x2333),
        (0x5234, 0xFFFF),
        (0x0101, 0x1523),
        (0xFFFF, 0xFFFF),
        (0x0101, 0x0101),
        (0x5235, 0x2700),
    ];

    let instrs: Vec<u8> = vec![0x09, 0x19, 0x29, 0x39];
    let instrs = instrs.into_iter().zip(common::REGISTERS_16B_NO_AF);

    let mut cpu = Cpu::new();
    for (x, y) in test_cases {
        for (op, reg) in instrs.clone() {
            cpu.registers.set_r16(Register16b::HL, *x);
            cpu.registers.set_r16(*reg, *y);

            cpu.execute_instr(op);

            match reg {
                Register16b::HL => {
                    assert_eq!(
                        y.wrapping_add(*y),
                        cpu.registers.get_r16(Register16b::HL),
                        "Op: 0x{:X}. Assertion failed for ADD HL, HL for operand 0x{:X})",
                        op,
                        y
                    );
                }
                _ => {
                    assert_eq!(
                        x.wrapping_add(*y),
                        cpu.registers.get_r16(Register16b::HL),
                        "Op: 0x{:X}. Assertion failed for ADD HL, {:?} for operands (0x{:X}, 0x{:X})",
                        op,
                        reg,
                        x,
                        y
                    );
                }
            }
            cpu.registers.set_r16(Register16b::HL, 0x0000);
        }
    }
}

#[test]
fn cpu_instr_sub_bytes() {
    let test_cases = &[
        // A    r8
        (0x00, 0x00),
        (0x23, 0x33),
        (0x54, 0xFF),
        (0x01, 0x15),
        (0xFF, 0xFF),
        (0x01, 0x01),
        (0x55, 0x27),
        (0x15, 0xFF),
        (0x12, 0x55),
    ];

    let instrs: Vec<u8> = vec![0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x97];
    let instrs = instrs.into_iter().zip(common::REGISTERS_8B_NO_F);

    let mut cpu = Cpu::new();
    for (x, y) in test_cases {
        for (op, reg) in instrs.clone() {
            cpu.registers.set_r8(Register8b::A, *x);
            cpu.registers.set_r8(*reg, *y);

            cpu.execute_instr(op);

            match reg {
                Register8b::A => {
                    assert_eq!(
                        y.wrapping_sub(*y),
                        cpu.registers.get_r8(Register8b::A),
                        "Op: 0x{:X}. Assertion failed for SUB A, A for operand 0x{:X})",
                        op,
                        y
                    );
                }
                _ => {
                    assert_eq!(
                        x.wrapping_sub(*y),
                        cpu.registers.get_r8(Register8b::A),
                        "Op: 0x{:X}. Assertion failed for SUB A, {:?} for operands (0x{:X}, 0x{:X})",
                        op,
                        reg,
                        x,
                        y
                    );
                }
            }
        }
    }
}

#[test]
fn cpu_instr_sbc_bytes() {
    let test_cases = &[
        // A    r8
        (0x00, 0x00),
        (0x23, 0x33),
        (0x54, 0xFF),
        (0x01, 0x15),
        (0xFF, 0xFF),
        (0x01, 0x01),
        (0x55, 0x27),
    ];

    let instrs: Vec<u8> = vec![0x98, 0x99, 0x9A, 0x9B, 0x9C, 0x9D, 0x9F];
    let instrs = instrs.into_iter().zip(common::REGISTERS_8B_NO_F);

    let mut cpu = Cpu::new();
    for (x, y) in test_cases {
        for (op, reg) in instrs.clone() {
            cpu.registers.set_r8(Register8b::A, *x);
            cpu.registers.set_r8(*reg, *y);
            cpu.registers.set_flag(Flag::C, true);

            cpu.execute_instr(op);

            match reg {
                Register8b::A => {
                    assert_eq!(
                        y.wrapping_sub(*y).wrapping_sub(1),
                        cpu.registers.get_r8(Register8b::A),
                        "Assertion failed for SBC A, A for operand 0x{:X})",
                        y
                    );
                }
                _ => {
                    assert_eq!(
                        x.wrapping_sub(*y).wrapping_sub(1),
                        cpu.registers.get_r8(Register8b::A),
                        "Assertion failed for SBC A, {:?} for operands (0x{:X}, 0x{:X})",
                        reg,
                        x,
                        y
                    );
                }
            }
        }
    }
}

#[test]
fn cpu_instr_and_a() {
    let test_cases = &[
        // A    r8
        (0x00, 0x00),
        (0x23, 0x33),
        (0x54, 0xFF),
        (0x01, 0x15),
        (0xFF, 0xFF),
        (0x01, 0x01),
        (0x55, 0x27),
    ];

    let instrs: Vec<u8> = vec![0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0xA7];
    let instrs = instrs.into_iter().zip(common::REGISTERS_8B_NO_F);

    let mut cpu = Cpu::new();
    for (a, y) in test_cases {
        for (op, reg) in instrs.clone() {
            cpu.registers.set_r8(Register8b::A, *a);
            cpu.registers.set_r8(*reg, *y);

            cpu.execute_instr(op);

            match reg {
                Register8b::A => {
                    assert_eq!(
                        *y,
                        cpu.registers.get_r8(Register8b::A),
                        "Op: 0x{:X}. Assertion failed for AND A, A for operand 0x{:X})",
                        op,
                        y
                    );
                }
                _ => {
                    assert_eq!(
                        a & y,
                        cpu.registers.get_r8(Register8b::A),
                        "Op: 0x{:X}. Assertion failed for AND A, {:?} for operands (0x{:X}, 0x{:X})",
                        op,
                        reg,
                        a,
                        y
                    );
                }
            }
            cpu.registers.set_r8(Register8b::A, 0x00);
        }
    }
}

#[test]
fn cpu_instr_xor_a() {
    let test_cases = &[
        // A    r8
        (0x00, 0x00),
        (0x23, 0x33),
        (0x54, 0xFF),
        (0x01, 0x15),
        (0xFF, 0xFF),
        (0x01, 0x01),
        (0x55, 0x27),
    ];

    let instrs: Vec<u8> = vec![0xA8, 0xA9, 0xAA, 0xAB, 0xAC, 0xAD, 0xAF];
    let instrs = instrs.into_iter().zip(common::REGISTERS_8B_NO_F);

    let mut cpu = Cpu::new();
    for (a, y) in test_cases {
        for (op, reg) in instrs.clone() {
            cpu.registers.set_r8(Register8b::A, *a);
            cpu.registers.set_r8(*reg, *y);

            cpu.execute_instr(op);

            match reg {
                Register8b::A => {
                    assert_eq!(
                        0,
                        cpu.registers.get_r8(Register8b::A),
                        "Op: 0x{:X}. Assertion failed for XOR A, A for operand 0x{:X})",
                        op,
                        y
                    );
                }
                _ => {
                    assert_eq!(
                        a ^ y,
                        cpu.registers.get_r8(Register8b::A),
                        "Op: 0x{:X}. Assertion failed for XOR A, {:?} for operands (0x{:X}, 0x{:X})",
                        op,
                        reg,
                        a,
                        y
                    );
                }
            }
            cpu.registers.set_r8(Register8b::A, 0x00);
        }
    }
}

#[test]
fn cpu_instr_or_a() {
    let test_cases = &[
        // A    r8
        (0x00, 0x00),
        (0x23, 0x33),
        (0x54, 0xFF),
        (0x01, 0x15),
        (0xFF, 0xFF),
        (0x01, 0x01),
        (0x55, 0x27),
    ];

    let instrs: Vec<u8> = vec![0xB0, 0xB1, 0xB2, 0xB3, 0xB4, 0xB5, 0xB7];
    let instrs = instrs.into_iter().zip(common::REGISTERS_8B_NO_F);

    let mut cpu = Cpu::new();
    for (a, y) in test_cases {
        for (op, reg) in instrs.clone() {
            cpu.registers.set_r8(Register8b::A, *a);
            cpu.registers.set_r8(*reg, *y);

            cpu.execute_instr(op);

            match reg {
                Register8b::A => {
                    assert_eq!(
                        *y,
                        cpu.registers.get_r8(Register8b::A),
                        "Op: 0x{:X}. Assertion failed for OR A, A for operand 0x{:X})",
                        op,
                        y
                    );
                }
                _ => {
                    assert_eq!(
                        a | y,
                        cpu.registers.get_r8(Register8b::A),
                        "Op: 0x{:X}. Assertion failed for OR A, {:?} for operands (0x{:X}, 0x{:X})",
                        op,
                        reg,
                        a,
                        y
                    );
                }
            }
            cpu.registers.set_r8(Register8b::A, 0x00);
        }
    }
}

#[test]
fn cpu_instr_cp_a() {
    let test_cases = &[
        // A    r8
        (0x00, 0x00),
        (0x23, 0x33),
        (0x54, 0xFF),
        (0x01, 0x15),
        (0xFF, 0xFF),
        (0x01, 0x01),
        (0x55, 0x27),
    ];

    let instrs: Vec<u8> = vec![0xB8, 0xB9, 0xBA, 0xBB, 0xBC, 0xBD, 0xBF];
    let instrs = instrs.into_iter().zip(common::REGISTERS_8B_NO_F);

    let mut cpu = Cpu::new();
    for (a, y) in test_cases {
        for (op, reg) in instrs.clone() {
            cpu.registers.set_r8(Register8b::A, *a);
            cpu.registers.set_r8(*reg, *y);

            cpu.execute_instr(op);

            match reg {
                Register8b::A => {
                    assert_eq!(
                        true, // flag Z is true if y == a
                        cpu.registers.flag_value(Flag::Z),
                        "Op: 0x{:X}. Assertion failed for CP A, A for operand 0x{:X})",
                        op,
                        y
                    );
                }
                _ => {
                    assert_eq!(
                        a < y, // flag C is true if a < y.
                        cpu.registers.flag_value(Flag::C),
                        "Op: 0x{:X}. Assertion failed for CP A, {:?} for operands (0x{:X}, 0x{:X})",
                        op,
                        reg,
                        a,
                        y
                    );
                    if a == y {
                        assert!(cpu.registers.flag_value(Flag::Z));
                    }
                }
            }
            cpu.registers.set_r8(Register8b::A, 0x00);
        }
    }
}
