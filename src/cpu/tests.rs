use super::*;

// test setup functions
mod common {
    use super::super::*;

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
    pub fn assert_flags(cpu: &Cpu, values: [bool; 4]) -> Result<(), &str> {
        let flags = vec![Flag::Z, Flag::N, Flag::H, Flag::C];
        let flags_str = vec!["Z (zero)", "N (subtraction)", "H (half-carry)", "C (carry)"];

        for (i, flag) in flags.into_iter().enumerate() {
            if cpu.registers.get_flag(flag) != values[i] {
                return Result::Err(flags_str[i]);
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
        let result = assert_flags(&cpu, values);

        if let Err(e) = result {
            let a = operands.0;
            let b = operands.1;
            panic!(
                "Flag assertion failed for {} with operands (0x{:X}, 0x{:X})",
                e, a, b
            );
        }
    }
}

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

// This is an integration test... requies MMU. not sure where to move??
#[test]
fn cpu_ld_r8_r8_instructions() {
    let mut cpu = Cpu::new();

    let registers_8b = vec![
        Register8b::B,
        Register8b::C,
        Register8b::D,
        Register8b::E,
        Register8b::H,
        Register8b::L,
        Register8b::A,
    ];

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
        cpu.registers.set_8b_reg(reg_from, test_value);
        // call CPU to load register from one to the other
        cpu.fetch_and_execute();
        // check that to_reg now has the same value
        assert_eq!(
            test_value,
            cpu.registers.get_8b_reg(reg_to),
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
        cpu.registers.set_8b_reg(reg_to, 0);
        cpu.registers.set_8b_reg(reg_from, 0);
    }
}

#[test]
fn cpu_ld_word_instructions() {}
