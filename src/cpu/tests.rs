use super::*;

// test setup functions
mod common {
    use super::super::*;

    pub fn registers_8b_no_f() -> Vec<Register8b> {
        vec![
            Register8b::B,
            Register8b::C,
            Register8b::D,
            Register8b::E,
            Register8b::H,
            Register8b::L,
            Register8b::A,
        ]
    }
    pub fn registers_16b_no_af() -> Vec<Register16b> {
        vec![
            Register16b::BC, Register16b::DE, Register16b::HL, Register16b::SP
        ]
    }

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
    pub fn assert_flags(cpu: &Cpu, values: Vec<bool>, flags: Vec<Flag>) -> Result<(), Flag> {
        let mut flag_strs: Vec<String> = Vec::new();

        for f in flags.clone() {
            flag_strs.push(format!("{:?}", f));
        }

        for (i, flag) in flags.into_iter().enumerate() {
            if cpu.registers.get_flag(flag) != values[i] {
                return Result::Err(flag);
            }
        }

        Result::Ok(())
    }

    /// Assert flags for binary operation with bytes (u8)
    ///
    /// # Arguments
    ///
    /// * `cpu`
    /// * `values` - Vector specifying values of flags in order of Flag::{Z, N, H, C}
    /// * `operands` - Tuple of two `u8`s of the binary operation being tested for
    pub fn assert_flags_binop(cpu: &Cpu, values: [bool; 4], operands: (u8, u8)) {
        let values = values.to_vec();

        let flags = vec![Flag::Z, Flag::N, Flag::H, Flag::C];
        let result = assert_flags(&cpu, values, flags);

        if let Err(f) = result {
            panic!(
                "Flag assertion failed for {:?} with operands (0x{:X}, 0x{:X})",
                f, operands.0, operands.1
            );
        }
    }

    pub fn assert_flags_unaop(cpu: &Cpu, values: [bool; 4], operand: u8) {
        let values = values.to_vec();

        let flags = vec![Flag::Z, Flag::N, Flag::H, Flag::C];
        let result = assert_flags(&cpu, values, flags);

        if let Err(f) = result {
            panic!(
                "Flag assertion failed for {:?} with operand {:X}",
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

    let test_cases: Vec<(u8, u8)> = vec![
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

    for (a, b) in &test_cases {
        let result = cpu.alu_add_bytes(*a, *b, false);
        assert_eq!(a.wrapping_add(*b), result);
    }
}

#[test]
fn cpu_alu_add_bytes_flag_z() {
    let mut cpu = Cpu::new();

    let test_cases = vec![
        // a    b       Z     N      H      C
        ((0x00, 0x00), [true, false, false, false]),
        ((0xFF, 0x01), [true, false, true, true]),
        ((0xFE, 0x02), [true, false, true, true]),
        ((0x02, 0xFE), [true, false, true, true]),
    ];

    for ((a, b), flags) in test_cases {
        let result: u8 = cpu.alu_add_bytes(a, b, false);
        assert_eq!(0, result);

        common::assert_flags_binop(&cpu, flags, (a, b));
    }
}

#[test]
fn cpu_alu_add_bytes_flag_h() {
    let mut cpu = Cpu::new();

    let test_cases = vec![
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
        let result: u8 = cpu.alu_add_bytes(a, b, false);
        assert_eq!(a.wrapping_add(b), result); // unnecessary?

        common::assert_flags_binop(&cpu, flags, (a, b));
    }
}

#[test]
fn cpu_alu_add_bytes_flag_c() {
    let mut cpu = Cpu::new();

    let test_cases = vec![
        // a    b       Z     N      H      C
        ((0x00, 0x00), [true, false, false, false]),
        ((0x00, 0x01), [false, false, false, false]),
        ((0xFF, 0x01), [true, false, true, true]),
        ((0xFE, 0x02), [true, false, true, true]),
        ((0xF0, 0x20), [false, false, false, true]),
    ];

    for ((a, b), flags) in test_cases {
        let result: u8 = cpu.alu_add_bytes(a, b, false);
        assert_eq!(a.wrapping_add(b), result);

        common::assert_flags_binop(&cpu, flags, (a, b));
    }
}

#[test]
fn cpu_alu_add_bytes_with_carry() {
    let mut cpu = Cpu::new();
    let test_cases = vec![
        // a    b       Z     N      H      C
        ((0x00, 0x00), [false, false, false, false]),
    ];

    for ((a, b), flags) in test_cases {
        cpu.registers.set_flag(Flag::C, true);
        let result = cpu.alu_add_bytes(a, b, true);
        assert_eq!(a.wrapping_add(b).wrapping_add(1), result);

        common::assert_flags_binop(&cpu, flags, (a, b));
    }
}

#[test]
fn cpu_alu_sub_bytes_value() {
    let mut cpu = Cpu::new();

    let test_cases = vec![
        (0x00, 0x00),
        (0x01, 0x00),
        (0xFF, 0x32),
        (0x01, 0x32),
        (0x32, 0x32),
    ];

    for (a, b) in test_cases {
        let result = cpu.alu_sub_bytes(a, b, false);
        assert_eq!(a.wrapping_sub(b), result);
    }
}

#[test]
fn cpu_alu_sub_bytes_flag_z() {
    let mut cpu = Cpu::new();

    // NOTE: Let x - y == 0 => x == y.
    //  shouldn't be possible to underflow into 0 if both are u8?
    //  similar reasonining for why H should never be set if x == y.
    let test_cases = vec![
        // a    b       Z     N      H      C
        ((0x00, 0x00), [true, true, false, false]),
        ((0x01, 0x01), [true, true, false, false]),
        ((0x1F, 0x1F), [true, true, false, false]),
        ((0xFF, 0xFF), [true, true, false, false]),
    ];

    for ((a, b), flags) in test_cases {
        let result: u8 = cpu.alu_sub_bytes(a, b, false);
        assert_eq!(a.wrapping_sub(b), result);

        common::assert_flags_binop(&cpu, flags, (a, b));
    }
}

#[test]
fn cpu_alu_sub_bytes_flag_h() {
    let mut cpu = Cpu::new();

    let test_cases = vec![
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
        let result: u8 = cpu.alu_sub_bytes(a, b, false);
        assert_eq!(a.wrapping_sub(b), result);

        common::assert_flags_binop(&cpu, flags, (a, b));
    }
}

#[test]
fn cpu_alu_sub_bytes_flag_c() {
    let mut cpu = Cpu::new();

    let test_cases = vec![
        // a    b       Z     N      H      C
        ((0xFE, 0xFF), [false, true, true, true]),
        ((0x00, 0x01), [false, true, true, true]),
    ];

    for ((a, b), flags) in test_cases {
        let result: u8 = cpu.alu_sub_bytes(a, b, false);
        assert_eq!(a.wrapping_sub(b), result);

        common::assert_flags_binop(&cpu, flags, (a, b));
    }
}

#[test]
fn cpu_alu_sub_bytes_with_carry() {
    let mut cpu = Cpu::new();

    let test_cases = vec![
        //  a   b
        (0x01, 0x00),
        (0x01, 0x01),
        (0x10, 0x02),
        (0x10, 0x1F),
    ];

    for (a, b) in test_cases {
        cpu.registers.set_flag(Flag::C, true);
        let result: u8 = cpu.alu_sub_bytes(a, b, true);

        assert_eq!(a.wrapping_sub(b).wrapping_sub(1), result);
    }
}

#[test]
fn cpu_alu_inc_byte() {
    let mut cpu = Cpu::new();

    let test_cases = vec![
        //      Z      N      H
        (0xFE, [false, false, false]),
    ];

    let flags = vec![Flag::Z, Flag::N, Flag::H];
    for (a, flag_vals) in test_cases {
        let value = cpu.alu_inc_byte(a);

        assert_eq!(value, a.wrapping_add(1));

        // check flag assertion
        let result = common::assert_flags(&cpu, flag_vals.to_vec(), flags.clone());
        if let Err(f) = result {
            panic!("Flag assertion failed for {:?} with value {:X}.", f, a);
        }
    }
}

#[test]
fn cpu_alu_dec_byte() {
    let mut cpu = Cpu::new();

    let test_cases = vec![
        //        Z      N     H
        (0xFF, [false, true, false]),
        (0x00, [false, true, true]),
        (0x9F, [false, true, false]),
        (0x13, [false, true, false]),
        (0x01, [true, true, false]),
        (0x0F, [false, true, false]),
        (0x10, [false, true, true]),
    ];

    let flags = vec![Flag::Z, Flag::N, Flag::H];
    for (a, flag_vals) in test_cases {
        let value = cpu.alu_dec_byte(a);

        assert_eq!(value, a.wrapping_sub(1));

        // check flag assertion
        let result = common::assert_flags(&cpu, flag_vals.to_vec(), flags.clone());
        if let Err(f) = result {
            panic!("Flag assertion failed for {:?} with value {:X}.", f, a);
        }
    }
}

#[test]
fn cpu_alu_add_words() {
    let mut cpu = Cpu::new();

    let test_cases = vec![
        //                N      H      C
        (0x0000, 0x0000, [false, false, false]),
        (0x1523, 0x2333, [false, false, false]),
        (0x0100, 0x0F00, [false, true, false]),
        (0x1000, 0xF000, [false, false, true]),
        (0x0100, 0xFF00, [false, true, true]),
        (0x0101, 0x1010, [false, false, false]),
        (0x0FFF, 0x0001, [false, true, false]),
    ];

    let flags = vec![Flag::N, Flag::H, Flag::C];
    for (x, y, flag_vals) in test_cases {
        let value = cpu.alu_add_words(x, y);
        assert_eq!(value, x.wrapping_add(y));

        let result = common::assert_flags(&cpu, flag_vals.to_vec(), flags.clone());
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

    let test_cases = vec![
        // a   y     Z     N     H     C
        (0x00, 0x00, [true, false, true, false]),
        (0x00, 0xFF, [true, false, true, false]),
        (0xFF, 0x00, [true, false, true, false]),
        (0x1F, 0x00, [true, false, true, false]),
        (0x11, 0x22, [true, false, true, false]), // TODO more test cases needed
    ];

    for (a, y, flags) in test_cases {
        cpu.registers.set_r8(Register8b::A, a);
        cpu.alu_and_a(y);

        assert_eq!(a & y, cpu.registers.get_r8(Register8b::A));
        common::assert_flags_binop(&cpu, flags, (a, y));
    }
}

#[test]
fn cpu_alu_xor_a() {
    let mut cpu = Cpu::new();

    let test_cases = vec![
        // a   y     Z     N     H     C
        (0x00, 0x00, [true, false, false, false]),
        (0xFF, 0xFF, [true, false, false, false]),
        (0xF0, 0x0F, [false, false, false, false]),
        (0xAA, 0x55, [false, false, false, false]),
    ];

    for (a, y, flags) in test_cases {
        cpu.registers.set_r8(Register8b::A, a);
        cpu.alu_xor_a(y);

        assert_eq!(a ^ y, cpu.registers.get_r8(Register8b::A));
        common::assert_flags_binop(&cpu, flags, (a, y));
    }
}

#[test]
fn cpu_alu_or_a() {
    let mut cpu = Cpu::new();

    let test_cases = vec![
        // a   y     Z     N     H     C
        (0x00, 0x00, [true, false, false, false]), // TODO more test cases needed
        (0xFF, 0xFF, [false, false, false, false]),
        (0xF0, 0x0F, [false, false, false, false]),
        (0xAA, 0x55, [false, false, false, false]),
        (0x00, 0xF0, [false, false, false, false]),
    ];

    for (a, y, flags) in test_cases {
        cpu.registers.set_r8(Register8b::A, a);
        cpu.alu_or_a(y);

        assert_eq!(a | y, cpu.registers.get_r8(Register8b::A));
        common::assert_flags_binop(&cpu, flags, (a, y));
    }
}

#[test]
fn cpu_alu_cp_a() {
    // flag tests should be covered by cpu_alu_sub_bytes
    let mut cpu = Cpu::new();

    let test_cases = vec![
        // a   y     Z     N     H     C
        (0x00, 0x00, [true, true, false, false]),
    ];

    for (a, y, flags) in test_cases {
        cpu.registers.set_r8(Register8b::A, a);
        cpu.alu_cp_a(y);

        assert_eq!(a, cpu.registers.get_r8(Register8b::A));
        common::assert_flags_binop(&cpu, flags, (a, y));
    }
}
/// TODO: move integration test
#[test]
fn cpu_fetch_byte() {
    let mut cpu = Cpu::new();

    let test_rom: Vec<u8> = vec![
        0x00, 0x50, 0x12, 0x55
    ];
    cpu.mmu.load_rom(test_rom.clone());

    cpu.registers.pc = 0;
    let mut test_pc: u16 = 0;
    for v in test_rom {
        assert_eq!(v, cpu.fetch_byte());
        test_pc += 1;
        assert_eq!(test_pc, cpu.registers.pc);
    }
}

/// TODO: move integration test
#[test]
fn cpu_fetch_word() {
    let mut cpu = Cpu::new();

    let test_vals: Vec<u16> = vec![
        0x0000, 0x5012, 0x12AE, 0x5EE5, 0xFFFF, 0x1283,
    ];

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

// This is an integration test... requies MMU. not sure where to move??
// This works for now. Consider rewriting this using "Cpu::execute_instr() if Mmu interface changes".
#[test]
fn cpu_instr_ld_r8_r8() {
    let mut cpu = Cpu::new();

    let registers_8b = common::registers_8b_no_f();

    // generate all LD test cases
    let mut test_cases: Vec<(u8, Register8b, Register8b)> = Vec::new();
    let mut op_code: u8 = 0x40;
    let mut ops: Vec<u8> = Vec::new();
    for reg_to in registers_8b.clone() {
        for reg_from in registers_8b.clone() {
            // println!("(0x{:X}, {:?}, {:?})", op_code, reg_from, reg_to);
            test_cases.push((op_code, reg_to, reg_from));
            ops.push(op_code);

            op_code += 1;

            if op_code % 8 == 6 {
                op_code += 1;
            } else if op_code == 0x70 {
                op_code += 8;
            }
        }
    }

    // init test rom and register verification variables
    //  test rom consists of all LD X, X ops in order, where X is a 8b register, excluding F
    let mut test_pc = 0x00;
    cpu.registers.pc = 0x00;
    cpu.mmu.load_rom(ops); // NOTE: unstable API for rom loading! subject to change

    let test_value: u8 = 0x0F;
    for (op, reg_to, reg_from) in test_cases {
        // set from_reg to desired value
        cpu.registers.set_r8(reg_from, test_value);
        // call CPU to load register from one to the other
        cpu.fetch_and_execute();
        // check that to_reg now has the same value
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
fn cpu_instr_ld_word() {
    //todo!("Not implemented yet!");
}

#[test]
fn cpu_instr_inc_byte() {
    let test_cases: Vec<u8> = vec![0x00, 0x01, 0x10, 0x0F, 0xFF];

    let mut cpu = Cpu::new();

    let r8s = common::registers_8b_no_f();
    let instrs: Vec<u8> = vec![
        0x04, // B
        0x0C, // C
        0x14, // D
        0x1C, // E
        0x24, // H
        0x2C, // L
        0x3C, // A
    ];

    let instrs = instrs.into_iter().zip(r8s);

    for test_value in test_cases {
        for (op, reg) in instrs.clone() {
            cpu.registers.set_r8(reg, test_value);
            cpu.execute_instr(op);

            assert_eq!(cpu.registers.get_r8(reg), test_value.wrapping_add(1));
        }
    }
}

#[test]
fn cpu_instr_inc_word() {
    let test_cases: Vec<u16> = vec![
        0x0032, 0x0000, 0x0001, 0xFFFF, 0xFF32, 0x5050, 0x2312,
    ];

    let r16s = common::registers_16b_no_af();
    let instrs: Vec<u8> = vec![0x03, 0x13, 0x23, 0x33];
    let instrs = instrs.into_iter().zip(r16s);

    let mut cpu = Cpu::new();
    for x in test_cases {
        for (op, reg) in instrs.clone() {
            cpu.registers.set_r16(reg, x);
            cpu.execute_instr(op);

            assert_eq!(x.wrapping_add(1), cpu.registers.get_r16(reg),
                "Op: 0x{:X}. {:?}, 0x{:X}", op, reg, x);
            cpu.registers.set_r16(reg, 0x0000);
        }
    }
}

#[test]
fn cpu_instr_dec_byte() {
    let test_cases: Vec<u8> = vec![0x00, 0x01, 0x10, 0x0F, 0xFF];

    let r8s = common::registers_8b_no_f();
    let instrs: Vec<u8> = vec![
        0x05, // B
        0x0D, // C
        0x15, // D
        0x1D, // E
        0x25, // H
        0x2D, // L
        0x3D, // A
    ];
    let instrs = instrs.into_iter().zip(r8s);

    let mut cpu = Cpu::new();
    for test_value in test_cases {
        for (op, reg) in instrs.clone() {
            cpu.registers.set_r8(reg, test_value);
            cpu.execute_instr(op);

            assert_eq!(cpu.registers.get_r8(reg), test_value.wrapping_sub(1));
        }
    }
}

#[test]
fn cpu_instr_dec_word() {
    let test_cases: Vec<u16> = vec![
        0x0032, 0x0000, 0x0001, 0xFFFF, 0xFF32, 0x5050, 0x2312,
    ];

    let r16s = common::registers_16b_no_af();
    let instrs: Vec<u8> = vec![0x0B, 0x1B, 0x2B, 0x3B];
    let instrs = instrs.into_iter().zip(r16s);

    let mut cpu = Cpu::new();
    for x in test_cases {
        for (op, reg) in instrs.clone() {
            cpu.registers.set_r16(reg, x);
            cpu.execute_instr(op);

            assert_eq!(x.wrapping_sub(1), cpu.registers.get_r16(reg),
                "Op: 0x{:X}. {:?}, 0x{:X}", op, reg, x);
            cpu.registers.set_r16(reg, 0x0000);
        }
    }
}

#[test]
fn cpu_instr_add_bytes() {
    let test_cases: Vec<(u8, u8)> = vec![
        // A    r8
        (0x00, 0x00),
        (0x23, 0x33),
        (0x54, 0xFF),
        (0x01, 0x15),
        (0xFF, 0xFF),
        (0x01, 0x01),
        (0x55, 0x27),
    ];

    let r8s = common::registers_8b_no_f();
    let instrs: Vec<u8> = vec![0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x87];
    let instrs = instrs.into_iter().zip(r8s);

    let mut cpu = Cpu::new();
    for (x, y) in test_cases {
        for (op, reg) in instrs.clone() {
            cpu.registers.set_r8(Register8b::A, x);
            cpu.registers.set_r8(reg, y);

            cpu.execute_instr(op);

            match reg {
                Register8b::A => {
                    assert_eq!(
                        y.wrapping_add(y),
                        cpu.registers.get_r8(Register8b::A),
                        "Op: 0x{:X}. Assertion failed for ADD A, A for operand 0x{:X})",
                        op,
                        y
                    );
                }
                _ => {
                    assert_eq!(
                        x.wrapping_add(y),
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
    let test_cases: Vec<(u8, u8)> = vec![
        // A    r8,  carry
        (0x00, 0x00),
        (0x23, 0x33),
        (0x54, 0xFF),
        (0x01, 0x15),
        (0xFF, 0xFF),
        (0x01, 0x01),
        (0x55, 0x27),
    ];

    let r8s = common::registers_8b_no_f();
    let instrs: Vec<u8> = vec![0x88, 0x89, 0x8A, 0x8B, 0x8C, 0x8D, 0x8F];
    let instrs = instrs.into_iter().zip(r8s);

    let mut cpu = Cpu::new();
    for (x, y) in test_cases {
        for (op, reg) in instrs.clone() {
            cpu.registers.set_r8(Register8b::A, x);
            cpu.registers.set_r8(reg, y);
            cpu.registers.set_flag(Flag::C, true);

            cpu.execute_instr(op);

            match reg {
                Register8b::A => {
                    assert_eq!(
                        y.wrapping_add(y).wrapping_add(1),
                        cpu.registers.get_r8(Register8b::A),
                        "Op: 0x{:X}. Assertion failed for ADC A, A for operand 0x{:X})",
                        op,
                        y
                    );
                }
                _ => {
                    assert_eq!(
                        x.wrapping_add(y).wrapping_add(1),
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
    let test_cases: Vec<(u16, u16)> = vec![
        // HL    r16
        (0x0000, 0x0000),
        (0x1523, 0x2333),
        (0x5234, 0xFFFF),
        (0x0101, 0x1523),
        (0xFFFF, 0xFFFF),
        (0x0101, 0x0101),
        (0x5235, 0x2700),
    ];

    let r16s = common::registers_16b_no_af();
    let instrs: Vec<u8> = vec![0x09, 0x19, 0x29, 0x39];
    let instrs = instrs.into_iter().zip(r16s);

    let mut cpu = Cpu::new();
    for (x, y) in test_cases {
        for (op, reg) in instrs.clone() {
            cpu.registers.set_r16(Register16b::HL, x);
            cpu.registers.set_r16(reg, y);

            cpu.execute_instr(op);
            
            match reg {
                Register16b::HL => {
                    assert_eq!(
                        y.wrapping_add(y),
                        cpu.registers.get_r16(Register16b::HL),
                        "Op: 0x{:X}. Assertion failed for ADD HL, HL for operand 0x{:X})",
                        op,
                        y
                    );
                }
                _ => {
                    assert_eq!(
                        x.wrapping_add(y),
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
    let test_cases: Vec<(u8, u8)> = vec![
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

    let r8s = common::registers_8b_no_f();
    let instrs: Vec<u8> = vec![0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x97];
    let instrs = instrs.into_iter().zip(r8s);

    let mut cpu = Cpu::new();
    for (x, y) in test_cases {
        for (op, reg) in instrs.clone() {
            cpu.registers.set_r8(Register8b::A, x);
            cpu.registers.set_r8(reg, y);

            cpu.execute_instr(op);

            match reg {
                Register8b::A => {
                    assert_eq!(
                        y.wrapping_sub(y),
                        cpu.registers.get_r8(Register8b::A),
                        "Op: 0x{:X}. Assertion failed for SUB A, A for operand 0x{:X})",
                        op,
                        y
                    );
                }
                _ => {
                    assert_eq!(
                        x.wrapping_sub(y),
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
    let test_cases: Vec<(u8, u8)> = vec![
        // A    r8
        (0x00, 0x00),
        (0x23, 0x33),
        (0x54, 0xFF),
        (0x01, 0x15),
        (0xFF, 0xFF),
        (0x01, 0x01),
        (0x55, 0x27),
    ];

    let r8s = common::registers_8b_no_f();
    let instrs: Vec<u8> = vec![0x98, 0x99, 0x9A, 0x9B, 0x9C, 0x9D, 0x9F];
    let instrs = instrs.into_iter().zip(r8s);

    let mut cpu = Cpu::new();
    for (x, y) in test_cases {
        for (op, reg) in instrs.clone() {
            cpu.registers.set_r8(Register8b::A, x);
            cpu.registers.set_r8(reg, y);
            cpu.registers.set_flag(Flag::C, true);

            cpu.execute_instr(op);

            match reg {
                Register8b::A => {
                    assert_eq!(
                        y.wrapping_sub(y).wrapping_sub(1),
                        cpu.registers.get_r8(Register8b::A),
                        "Assertion failed for SBC A, A for operand 0x{:X})",
                        y
                    );
                }
                _ => {
                    assert_eq!(
                        x.wrapping_sub(y).wrapping_sub(1),
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
    let test_cases: Vec<(u8, u8)> = vec![
        // A    r8
        (0x00, 0x00),
        (0x23, 0x33),
        (0x54, 0xFF),
        (0x01, 0x15),
        (0xFF, 0xFF),
        (0x01, 0x01),
        (0x55, 0x27),
    ];

    let r8s = common::registers_8b_no_f();
    let instrs: Vec<u8> = vec![0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0xA7];
    let instrs = instrs.into_iter().zip(r8s);

    let mut cpu = Cpu::new();
    for (a, y) in test_cases {
        for (op, reg) in instrs.clone() {
            cpu.registers.set_r8(Register8b::A, a);
            cpu.registers.set_r8(reg, y);

            cpu.execute_instr(op);

            match reg {
                Register8b::A => {
                    assert_eq!(
                        y,
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
    let test_cases: Vec<(u8, u8)> = vec![
        // A    r8
        (0x00, 0x00),
        (0x23, 0x33),
        (0x54, 0xFF),
        (0x01, 0x15),
        (0xFF, 0xFF),
        (0x01, 0x01),
        (0x55, 0x27),
    ];

    let r8s = common::registers_8b_no_f();
    let instrs: Vec<u8> = vec![0xA8, 0xA9, 0xAA, 0xAB, 0xAC, 0xAD, 0xAF];
    let instrs = instrs.into_iter().zip(r8s);

    let mut cpu = Cpu::new();
    for (a, y) in test_cases {
        for (op, reg) in instrs.clone() {
            cpu.registers.set_r8(Register8b::A, a);
            cpu.registers.set_r8(reg, y);

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
    let test_cases: Vec<(u8, u8)> = vec![
        // A    r8
        (0x00, 0x00),
        (0x23, 0x33),
        (0x54, 0xFF),
        (0x01, 0x15),
        (0xFF, 0xFF),
        (0x01, 0x01),
        (0x55, 0x27),
    ];

    let r8s = common::registers_8b_no_f();
    let instrs: Vec<u8> = vec![0xB0, 0xB1, 0xB2, 0xB3, 0xB4, 0xB5, 0xB7];
    let instrs = instrs.into_iter().zip(r8s);

    let mut cpu = Cpu::new();
    for (a, y) in test_cases {
        for (op, reg) in instrs.clone() {
            cpu.registers.set_r8(Register8b::A, a);
            cpu.registers.set_r8(reg, y);

            cpu.execute_instr(op);

            match reg {
                Register8b::A => {
                    assert_eq!(
                        y,
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
    let test_cases: Vec<(u8, u8)> = vec![
        // A    r8
        (0x00, 0x00),
        (0x23, 0x33),
        (0x54, 0xFF),
        (0x01, 0x15),
        (0xFF, 0xFF),
        (0x01, 0x01),
        (0x55, 0x27),
    ];

    let r8s = common::registers_8b_no_f();
    let instrs: Vec<u8> = vec![0xB8, 0xB9, 0xBA, 0xBB, 0xBC, 0xBD, 0xBF];
    let instrs = instrs.into_iter().zip(r8s);

    let mut cpu = Cpu::new();
    for (a, y) in test_cases {
        for (op, reg) in instrs.clone() {
            cpu.registers.set_r8(Register8b::A, a);
            cpu.registers.set_r8(reg, y);

            cpu.execute_instr(op);

            match reg {
                Register8b::A => {
                    assert_eq!( 
                        true,   // flag Z is true if y == a
                        cpu.registers.get_flag(Flag::Z),
                        "Op: 0x{:X}. Assertion failed for CP A, A for operand 0x{:X})",
                        op,
                        y
                    );
                }
                _ => {
                    assert_eq!(
                        a < y,  // flag C is true if y > a.
                        cpu.registers.get_flag(Flag::C),
                        "Op: 0x{:X}. Assertion failed for CP A, {:?} for operands (0x{:X}, 0x{:X})",
                        op,
                        reg,
                        a,
                        y
                    );
                    if a == y {
                        assert!(cpu.registers.get_flag(Flag::Z));
                    }
                }
            }
            cpu.registers.set_r8(Register8b::A, 0x00);
        }
    }
}
