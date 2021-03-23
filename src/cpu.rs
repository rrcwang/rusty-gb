// modules
use crate::memory;
use crate::registers;
use crate::utils::{bytes_to_word, word_to_bytes};
// types
use registers::Flag;
use registers::Register16b;
use registers::Register8b;

pub struct Cpu {
    registers: registers::Registers,
    // CPU counters/states
    halted: bool,
    interrupt_master_enable: bool, // IME
    // memory
    mmu: memory::Mmu,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            // TODO: what are the initilization values here?
            registers: registers::Registers::new(),
            halted: false,
            interrupt_master_enable: false,
            mmu: memory::Mmu::new(),
        }
    }

    /// Adds two byte length values and sets the appropriate flags in the F register for CPU instructions
    ///
    /// Applicable for instructions 0x80..0x8F
    ///
    /// # Arguments
    /// `x`, `y` - Operands for the addition
    ///
    /// # Flags
    /// Z: true iff result == 0
    /// N: false, not subtraction/negative
    /// H: if sum of lower 4 bits overflows
    /// C: if sum of 8 bits overflows
    fn alu_add_bytes(&mut self, x: u8, y: u8, use_carry: bool) -> u8 {
        let c: u16 = match use_carry & self.registers.get_flag(Flag::C) {
            true => 1,
            false => 0,
        };
        let x = x as u16;
        let y = (y as u16).wrapping_add(c);

        let result = x.wrapping_add(y);

        // set flags
        // subtraction
        self.registers.set_flag(Flag::N, false);
        // half-carry: https://stackoverflow.com/questions/62006764/how-is-xor-applied-when-determining-carry
        //  x + y == x ^ y ^ carry_bits
        //  x ^ y ^ sum == carry_bits
        //  (x ^ y ^ sum) & 0x10 == carry_bits & 0x10 == carry out for bit 4
        self.registers
            .set_flag(Flag::H, ((x ^ y ^ result) & 0x10) != 0);
        // carry
        self.registers.set_flag(Flag::C, (result & 0x100) != 0);
        // cast to 8-bit
        let result = result as u8;
        // zero
        self.registers.set_flag(Flag::Z, result == 0);

        result // return 8-bit
    }

    /// Computes (x - y) and sets the relevant flags.
    ///
    /// # Arguments
    /// `x`, `y` - Operands for the subtraction
    fn alu_sub_bytes(&mut self, x: u8, y: u8, use_carry: bool) -> u8 {
        let c: u16 = match use_carry & self.registers.get_flag(Flag::C) {
            true => 1,
            false => 0,
        };
        let x = x as u16;
        let y = (y as u16).wrapping_add(c);

        let result = x.wrapping_sub(y);

        // set flags
        // subtraction
        self.registers.set_flag(Flag::N, true);
        // half-carry
        self.registers.set_flag(Flag::H, (y & 0x0F) > (x & 0x0F)); // x ^ (!y) ^ result ??? off-by-one
                                                                   // carry
        self.registers.set_flag(Flag::C, (result & 0x100) != 0); // two's complement subtraction. should work?
                                                                 // cast to 8-bit
        let result = result as u8;
        // zero
        self.registers.set_flag(Flag::Z, result == 0);

        result // return 8-bit
    }

    /// Adds two unsigned words, and sets the appropriate flags.
    ///
    /// Implemented as chaining two byte lenght adds together.
    /// Used in ADD HL, r16 instructions
    ///
    /// # Arguments
    /// * `(x, y)` - `u16`s
    ///
    /// # Output
    /// * Result of `x + y`
    ///
    /// # Flags
    /// * `N = 0`
    /// * `H` if bit 15 overflows
    /// * `C` if bit 11 overflows
    fn alu_add_words(&mut self, x: u16, y: u16) -> u16 {
        let (x_high, x_low) = word_to_bytes(x);
        let (y_high, y_low) = word_to_bytes(y);

        // need to leave z untouched, but alu_add_bytes modifies Z
        let z = self.registers.get_flag(Flag::Z);

        let low = self.alu_add_bytes(x_low, y_low, false);
        let high = self.alu_add_bytes(x_high, y_high, true);

        self.registers.set_flag(Flag::Z, z);

        bytes_to_word(high, low)
    }

    /// Increments the value of `x` by one.
    ///
    /// Wraps around if saturation is reached.
    ///
    /// # Argument
    /// * `x` - `u8` value
    ///
    /// # Flags
    /// * `Z` if result is 0 (equivalent to C)
    /// * `N = 0`
    /// * `H` if bit 3 overflows
    fn alu_inc_byte(&mut self, x: u8) -> u8 {
        let result = x.wrapping_add(1);

        self.registers.set_flag(Flag::Z, result == 0);
        self.registers.set_flag(Flag::N, false);
        self.registers.set_flag(Flag::H, (x & 0x0F) + 1 > 0x0F);

        result
    }

    /// Decrements the value of `x` by one.
    ///
    /// Wraps around if saturation is reached.
    ///
    /// # Argument
    /// * `x` - `u8` value
    ///
    /// # Flags
    /// * `Z` if result is 0
    /// * `N = 1`
    /// * `H` if bit 3 borrows
    fn alu_dec_byte(&mut self, x: u8) -> u8 {
        let result = x.wrapping_sub(1);

        self.registers.set_flag(Flag::Z, result == 0);
        self.registers.set_flag(Flag::N, true);
        self.registers.set_flag(Flag::H, (x & 0x0F) == 0);
        // half-carry condition, borrow from 4->3
        // 0xN0 - 0x01 == 0x(N-1)F <=> 0xN0 & 0x0F == 0
        result
    }

    fn alu_and_a(&mut self, y: u8) {
        let a = self.registers.get_r8(Register8b::A);

        let result = a & y;

        self.registers.set_flag(Flag::Z, result == 0);
        self.registers.set_flag(Flag::N, false);
        self.registers.set_flag(Flag::H, true);
        self.registers.set_flag(Flag::C, false);

        self.registers.set_r8(Register8b::A, result);
    }

    fn alu_xor_a(&mut self, y: u8) {
        let a = self.registers.get_r8(Register8b::A);

        let result = a ^ y;

        self.registers.set_flag(Flag::Z, result == 0);
        self.registers.set_flag(Flag::N, false);
        self.registers.set_flag(Flag::H, false);
        self.registers.set_flag(Flag::C, false);

        self.registers.set_r8(Register8b::A, result);
    }

    fn alu_or_a(&mut self, y: u8) {
        let a = self.registers.get_r8(Register8b::A);

        let result = a | y;

        self.registers.set_flag(Flag::Z, result == 0);
        self.registers.set_flag(Flag::N, false);
        self.registers.set_flag(Flag::H, false);
        self.registers.set_flag(Flag::C, false);

        self.registers.set_r8(Register8b::A, result);
    }

    fn alu_cp_a(&mut self, y: u8) {
        let a = self.registers.get_r8(Register8b::A);
        self.alu_sub_bytes(a, y, false);
    }

    /// Loads the value from one 8-bit register to another. Used for LD R8, R8 instructions.
    fn ld_regs_8b(&mut self, reg_to: Register8b, reg_from: Register8b) {
        /* if reg_from == reg_to { // ignore for now, assuming is not called for same reg instructions
            return
        } */
        let value = self.registers.get_r8(reg_from);
        self.registers.set_r8(reg_to, value);
    }

    /// Fetches the next instruction in the program and executes it
    /// returns the duration taken by the instruction in clock ticks taken
    ///
    /// # Return value
    /// `t_states: u8` - Number of clock ticks taken to run instruction.
    ///
    /// **NOTE:** one CPU cycle/"M-cycle" == four clock ticks/"T-states"
    pub fn fetch_and_execute(&mut self) -> u8 {
        // TODO: fetch OP code from ROM
        // 0. fetch next instruction, pointed to by PC
        // let instruction: u8 = 0x00;
        let instruction: u8 = self.mmu.read_byte(self.registers.pc);

        // DEBUG:
        println!(
            "CPU executing 0x{:X} at PC: {}",
            instruction, self.registers.pc
        );

        // increment past current opcode
        self.registers.pc += 1;

        // 1. decode and execute instruction
        self.execute_instr(instruction)
    }

    /// Executes CPU instruction.
    ///     - do thing
    ///     - set flags
    ///     - increment program counter past immediate (operands) if they exist
    ///     - return number of cycles
    ///  All op-codes/instructions documented here: https://gbdev.io/gb-opcodes/optables/
    ///
    /// # Arguments
    /// `instruction: u16` - Compiled machine code instruction for the CPU
    ///
    /// # Return value
    /// `t_states: u8` - Number of clock ticks taken to run instruction
    fn execute_instr(&mut self, instruction: u8) -> u8 {
        match instruction {
            // 0x00 -> 0x0F
            0x00 => 4, // NOP   | 0x00          | do nothing for 1 cycle
            0x01 => {
                // TOTEST
                // LD BC, d16   | 0x01 0xNNNN   | load into BC, 0xNNNN
                let value: u16 = self.mmu.read_word(self.registers.pc); // read data from program
                self.registers.pc = self.registers.pc.wrapping_add(2); // length of operands
                self.registers.set_r16(Register16b::BC, value);
                12 // program takes 12 T-states
            }
            0x02 => {
                // TOTEST
                // LD (BC), A   | 0x02          | write byte stored in A to memory location (BC)
                let value = self.registers.get_r8(Register8b::A);
                let address = self.registers.get_r16(Register16b::BC);
                self.mmu.write_byte(address, value);
                8
            }
            0x03 => {
                // TOTEST
                // INC BC      | 0x03           | increment BC register by 1
                let value = self.registers.get_r16(Register16b::BC);
                self.registers
                    .set_r16(Register16b::BC, value.wrapping_add(1));
                8
            }
            0x04 => {
                // INC B
                let mut value = self.registers.get_r8(Register8b::B);
                value = self.alu_inc_byte(value);
                self.registers.set_r8(Register8b::B, value);
                4
            }
            0x05 => {
                // DEC B
                let mut value = self.registers.get_r8(Register8b::B);
                value = self.alu_dec_byte(value);
                self.registers.set_r8(Register8b::B, value);
                4
            }
            0x06 => {
                // TOTEST
                // LD B, d8
                let value: u8 = self.mmu.read_byte(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
                self.registers.set_r8(Register8b::B, value);
                8
            }
            0x07 => {
                // TODO
                // RLCA
                self.unimpl_instr();
                self.registers.set_r8(Register8b::F, 0b_0000_0000);
                // TODO: set carry flag accordingly
                4
            }
            0x08 => {
                // LD (a16), SP | 0x03 0xNNNN    | write stack pointer, u16 to memory address in operand
                let address = self.mmu.read_word(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(2);
                self.mmu.write_word(address, self.registers.sp);
                20
            }
            0x09 => {
                // ADD HL, BC   | 0x09           | add BC to HL
                let hl = self.registers.get_r16(Register16b::HL);
                let bc = self.registers.get_r16(Register16b::BC);
                let result = self.alu_add_words(hl, bc);
                self.registers.set_r16(Register16b::HL, result);
                8
            }
            0x0A => {
                // TOTEST
                // LD A, (BC)
                let address = self.registers.get_r16(Register16b::BC);
                let value = self.mmu.read_byte(address);
                self.registers.set_r8(Register8b::A, value);
                8
            }
            0x0B => {
                // TOTEST
                // DEC BC
                let value = self.registers.get_r16(Register16b::BC);
                let value = value.wrapping_sub(1);
                // no flags set!
                self.registers.set_r16(Register16b::BC, value);
                8
            }
            0x0C => {
                // INC C
                let value = self.registers.get_r8(Register8b::C);
                let value = self.alu_inc_byte(value);
                self.registers.set_r8(Register8b::C, value);
                4
            }
            0x0D => {
                // DEC C
                let value = self.registers.get_r8(Register8b::C);
                let value = self.alu_dec_byte(value);
                self.registers.set_r8(Register8b::C, value);
                4
            }
            0x0E => {
                // LD C, d8
                let value = self.mmu.read_byte(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
                self.registers.set_r8(Register8b::C, value);
                8
            }
            0x0F => {
                // TODO
                // RLCA
                self.unimpl_instr();
                4
            }
            0x10 => {
                // TODO
                // STOP
                self.unimpl_instr();
                self.registers.pc = self.registers.pc.wrapping_add(1);
                4
            }
            0x11 => {
                // TOTEST: memory
                // LD DE, d16
                let value = self.mmu.read_word(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(2);
                self.registers.set_r16(Register16b::DE, value);
                12
            }
            0x12 => {
                // TODO
                // LD (DE), A
                self.unimpl_instr();
            }
            0x13 => {
                // INC DE
                let value = self.registers.get_r16(Register16b::DE);
                self.registers
                    .set_r16(Register16b::DE, value.wrapping_add(1));
                8
            }
            0x14 => {
                // INC D
                let value = self.registers.get_r8(Register8b::D);
                let value = self.alu_inc_byte(value);
                self.registers.set_r8(Register8b::D, value);
                4
            }
            0x15 => {
                // DEC D
                let value = self.registers.get_r8(Register8b::D);
                let value = self.alu_dec_byte(value);
                self.registers.set_r8(Register8b::D, value);
                4
            }
            0x19 => {
                // ADD HL, DE
                let hl = self.registers.get_r16(Register16b::HL);
                let de = self.registers.get_r16(Register16b::DE);
                let result = self.alu_add_words(hl, de);
                self.registers.set_r16(Register16b::HL, result);
                8
            }
            0x1B => {
                // DEC DE
                let value = self.registers.get_r16(Register16b::DE);
                self.registers
                    .set_r16(Register16b::DE, value.wrapping_sub(1));
                8
            }
            0x1C => {
                // INC E
                let value = self.registers.get_r8(Register8b::E);
                let value = self.alu_inc_byte(value);
                self.registers.set_r8(Register8b::E, value);
                4
            }
            0x1D => {
                // DEC E
                let value = self.registers.get_r8(Register8b::E);
                let value = self.alu_dec_byte(value);
                self.registers.set_r8(Register8b::E, value);
                4
            }
            // 0x20 -> 0x2F
            0x23 => {
                // INC HL
                let value = self.registers.get_r16(Register16b::HL);
                self.registers
                    .set_r16(Register16b::HL, value.wrapping_add(1));
                8
            }
            0x24 => {
                // INC H
                let value = self.registers.get_r8(Register8b::H);
                let value = self.alu_inc_byte(value);
                self.registers.set_r8(Register8b::H, value);
                4
            }
            0x25 => {
                // DEC H
                let value = self.registers.get_r8(Register8b::H);
                let value = self.alu_dec_byte(value);
                self.registers.set_r8(Register8b::H, value);
                4
            }
            0x29 => {
                // ADD HL, HL
                let hl = self.registers.get_r16(Register16b::HL);
                let result = self.alu_add_words(hl, hl);
                self.registers.set_r16(Register16b::HL, result);
                8
            }
            0x2B => {
                // DEC HL
                let value = self.registers.get_r16(Register16b::HL);
                self.registers
                    .set_r16(Register16b::HL, value.wrapping_sub(1));
                8
            }
            0x2C => {
                // INC L
                let value = self.registers.get_r8(Register8b::L);
                let value = self.alu_inc_byte(value);
                self.registers.set_r8(Register8b::L, value);
                4
            }
            0x2D => {
                // DEC L
                let value = self.registers.get_r8(Register8b::L);
                let value = self.alu_dec_byte(value);
                self.registers.set_r8(Register8b::L, value);
                4
            }
            // 0x30 -> 0x3F
            0x33 => {
                // INC SP
                self.registers.sp = self.registers.sp.wrapping_add(1);
                8
            }
            0x39 => {
                // ADD HL, SP
                let hl = self.registers.get_r16(Register16b::HL);
                let sp = self.registers.get_r16(Register16b::SP);
                let result = self.alu_add_words(hl, sp);
                self.registers.set_r16(Register16b::HL, result);
                8
            }
            0x3B => {
                // DEC SP
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                8
            }
            0x3C => {
                // INC A
                let value = self.registers.get_r8(Register8b::A);
                let value = self.alu_inc_byte(value);
                self.registers.set_r8(Register8b::A, value);
                4
            }
            0x3D => {
                // DEC A
                let value = self.registers.get_r8(Register8b::A);
                let value = self.alu_dec_byte(value);
                self.registers.set_r8(Register8b::A, value);
                4
            }
            // 0x40 -> 0x4F
            0x40 => {
                // LD B, B    | does nothing
                4
            }
            0x41 => {
                // LD B, C
                self.ld_regs_8b(Register8b::B, Register8b::C);
                4
            }
            0x42 => {
                // LD B, D
                self.ld_regs_8b(Register8b::B, Register8b::D);
                4
            }
            0x43 => {
                // LD B, E
                self.ld_regs_8b(Register8b::B, Register8b::E);
                4
            }
            0x44 => {
                // LD B, H
                self.ld_regs_8b(Register8b::B, Register8b::H);
                4
            }
            0x45 => {
                // LD B, L
                self.ld_regs_8b(Register8b::B, Register8b::L);
                4
            }
            0x46 => {
                // TODO
                // LD B, (HL)
                self.unimpl_instr();
                8
            }
            0x47 => {
                // LD B, A
                self.ld_regs_8b(Register8b::B, Register8b::A);
                4
            }
            0x48 => {
                // LD C, B
                self.ld_regs_8b(Register8b::C, Register8b::B);
                4
            }
            0x49 => {
                // LD C, C
                4
            }
            0x4A => {
                // LD C, D
                self.ld_regs_8b(Register8b::C, Register8b::D);
                4
            }
            0x4B => {
                // LD C, E
                self.ld_regs_8b(Register8b::C, Register8b::E);
                4
            }
            0x4C => {
                // LD C, H
                self.ld_regs_8b(Register8b::C, Register8b::H);
                4
            }
            0x4D => {
                // LD C, L
                self.ld_regs_8b(Register8b::C, Register8b::L);
                4
            }
            0x4E => {
                // TODO
                // LD C, (HL)
                self.unimpl_instr();
            }
            0x4F => {
                // LD C, A
                self.ld_regs_8b(Register8b::C, Register8b::A);
                4
            }
            // 0x50 -> 0x5F
            0x50 => {
                // LD D, B
                self.ld_regs_8b(Register8b::D, Register8b::B);
                4
            }
            0x51 => {
                // LD D, C
                self.ld_regs_8b(Register8b::D, Register8b::C);
                4
            }
            0x52 => {
                // LD D, D
                4
            }
            0x53 => {
                // LD D, E
                self.ld_regs_8b(Register8b::D, Register8b::E);
                4
            }
            0x54 => {
                // LD D, H
                self.ld_regs_8b(Register8b::D, Register8b::H);
                4
            }
            0x55 => {
                // LD D, L
                self.ld_regs_8b(Register8b::D, Register8b::L);
                4
            }
            0x56 => {
                // TODO
                // LD D, (HL)
                self.unimpl_instr();

                8
            }
            0x57 => {
                // LD D, A
                self.ld_regs_8b(Register8b::D, Register8b::A);
                4
            }
            0x58 => {
                // LD E, B
                self.ld_regs_8b(Register8b::E, Register8b::B);
                4
            }
            0x59 => {
                // LD E, C
                self.ld_regs_8b(Register8b::E, Register8b::C);
                4
            }
            0x5A => {
                // LD E, D
                self.ld_regs_8b(Register8b::E, Register8b::D);
                4
            }
            0x5B => {
                // LD E, E
                4
            }
            0x5C => {
                // LD E, H
                self.ld_regs_8b(Register8b::E, Register8b::H);
                4
            }
            0x5D => {
                // LD E, L
                self.ld_regs_8b(Register8b::E, Register8b::L);
                4
            }
            0x5E => {
                // TODO
                // LD E, (HL)
                self.unimpl_instr();
            }
            0x5F => {
                // LD E, A
                self.ld_regs_8b(Register8b::E, Register8b::A);
                4
            }
            // 0x60 -> 0x6F
            0x60 => {
                // LD H, B    | does nothing
                self.ld_regs_8b(Register8b::H, Register8b::B);
                4
            }
            0x61 => {
                // LD H, C
                self.ld_regs_8b(Register8b::H, Register8b::C);
                4
            }
            0x62 => {
                // LD H, D
                self.ld_regs_8b(Register8b::H, Register8b::D);
                4
            }
            0x63 => {
                // LD H, E
                self.ld_regs_8b(Register8b::H, Register8b::E);
                4
            }
            0x64 => {
                // LD H, H
                4
            }
            0x65 => {
                // LD H, L
                self.ld_regs_8b(Register8b::H, Register8b::L);
                4
            }
            0x66 => {
                // TODO
                // LD H, (HL)
                self.unimpl_instr();

                8
            }
            0x67 => {
                // LD H, A
                self.ld_regs_8b(Register8b::H, Register8b::A);
                4
            }
            0x68 => {
                // LD L, B
                self.ld_regs_8b(Register8b::L, Register8b::B);
                4
            }
            0x69 => {
                // LD L, C
                self.ld_regs_8b(Register8b::L, Register8b::C);
                4
            }
            0x6A => {
                // LD L, D
                self.ld_regs_8b(Register8b::L, Register8b::D);
                4
            }
            0x6B => {
                // LD L, E
                self.ld_regs_8b(Register8b::L, Register8b::E);
                4
            }
            0x6C => {
                // LD L, H
                self.ld_regs_8b(Register8b::L, Register8b::H);
                4
            }
            0x6D => {
                // LD L, L
                4
            }
            0x6E => {
                // TODO
                // LD L, (HL)
                self.unimpl_instr();
            }
            0x6F => {
                // LD L, A
                self.ld_regs_8b(Register8b::L, Register8b::A);
                4
            }
            // 0x70 -> 0x7F
            0x78 => {
                // LD A, B
                self.ld_regs_8b(Register8b::A, Register8b::B);
                4
            }
            0x79 => {
                // LD A, C
                self.ld_regs_8b(Register8b::A, Register8b::C);
                4
            }
            0x7A => {
                // LD A, D
                self.ld_regs_8b(Register8b::A, Register8b::D);
                4
            }
            0x7B => {
                // LD A, E
                self.ld_regs_8b(Register8b::A, Register8b::E);
                4
            }
            0x7C => {
                // LD A, H
                self.ld_regs_8b(Register8b::A, Register8b::H);
                4
            }
            0x7D => {
                // LD A, L
                self.ld_regs_8b(Register8b::A, Register8b::L);
                4
            }
            0x7E => {
                // TODO
                // LD A, (HL)
                self.unimpl_instr();
                8
            }
            0x7F => {
                // LD A, A
                4
            }
            // 0x80 -> 0x8F
            0x80 => {
                // ADD A, B
                let a = self.registers.get_r8(Register8b::A); // accumulator
                let y = self.registers.get_r8(Register8b::B); // operand
                let result = self.alu_add_bytes(a, y, false);

                self.registers.set_r8(Register8b::A, result);
                4
            }
            0x81 => {
                // ADD A, C
                let a = self.registers.get_r8(Register8b::A);
                let y = self.registers.get_r8(Register8b::C);
                let result = self.alu_add_bytes(a, y, false);

                self.registers.set_r8(Register8b::A, result);
                4
            }
            0x82 => {
                // ADD A, D
                let a = self.registers.get_r8(Register8b::A);
                let y = self.registers.get_r8(Register8b::D);
                let result = self.alu_add_bytes(a, y, false);

                self.registers.set_r8(Register8b::A, result);
                4
            }
            0x83 => {
                // ADD A, E
                let a = self.registers.get_r8(Register8b::A);
                let y = self.registers.get_r8(Register8b::E);
                let result = self.alu_add_bytes(a, y, false);

                self.registers.set_r8(Register8b::A, result);
                4
            }
            0x84 => {
                // ADD A, H
                let a = self.registers.get_r8(Register8b::A);
                let y = self.registers.get_r8(Register8b::H);
                let result = self.alu_add_bytes(a, y, false);

                self.registers.set_r8(Register8b::A, result);
                4
            }
            0x85 => {
                // ADD A, L
                let a = self.registers.get_r8(Register8b::A);
                let y = self.registers.get_r8(Register8b::L);
                let result = self.alu_add_bytes(a, y, false);

                self.registers.set_r8(Register8b::A, result);
                4
            }
            0x86 => {
                self.unimpl_instr();
            }
            0x87 => {
                // ADD A, A
                let a = self.registers.get_r8(Register8b::A);
                let result = self.alu_add_bytes(a, a, false);

                self.registers.set_r8(Register8b::A, result);
                4
            }
            0x88 => {
                // ADC A, B
                let a = self.registers.get_r8(Register8b::A);
                let y = self.registers.get_r8(Register8b::B);
                let result = self.alu_add_bytes(a, y, true);

                self.registers.set_r8(Register8b::A, result);
                4
            }
            0x89 => {
                // ADC A, C
                let a = self.registers.get_r8(Register8b::A);
                let y = self.registers.get_r8(Register8b::C);
                let result = self.alu_add_bytes(a, y, true);

                self.registers.set_r8(Register8b::A, result);
                4
            }
            0x8A => {
                // ADC A, D
                let a = self.registers.get_r8(Register8b::A);
                let y = self.registers.get_r8(Register8b::D);
                let result = self.alu_add_bytes(a, y, true);

                self.registers.set_r8(Register8b::A, result);
                4
            }
            0x8B => {
                // ADC A, E
                let a = self.registers.get_r8(Register8b::A);
                let y = self.registers.get_r8(Register8b::E);
                let result = self.alu_add_bytes(a, y, true);

                self.registers.set_r8(Register8b::A, result);
                4
            }
            0x8C => {
                // ADC A, H
                let a = self.registers.get_r8(Register8b::A);
                let y = self.registers.get_r8(Register8b::H);
                let result = self.alu_add_bytes(a, y, true);

                self.registers.set_r8(Register8b::A, result);
                4
            }
            0x8D => {
                // ADC A, L
                let a = self.registers.get_r8(Register8b::A);
                let y = self.registers.get_r8(Register8b::L);
                let result = self.alu_add_bytes(a, y, true);

                self.registers.set_r8(Register8b::A, result);
                4
            }
            0x8E => {
                // ADC A, (HL)
                self.unimpl_instr();
            }
            0x8F => {
                // ADC A, A
                let a = self.registers.get_r8(Register8b::A);
                let result = self.alu_add_bytes(a, a, true);

                self.registers.set_r8(Register8b::A, result);
                4
            }
            // 0x90 -> 0x9F
            0x90 => {
                // SUB A, B
                let a = self.registers.get_r8(Register8b::A);
                let y = self.registers.get_r8(Register8b::B);
                let result = self.alu_sub_bytes(a, y, false);

                self.registers.set_r8(Register8b::A, result);
                4
            }
            0x91 => {
                // SUB A, C
                let a = self.registers.get_r8(Register8b::A);
                let y = self.registers.get_r8(Register8b::C);
                let result = self.alu_sub_bytes(a, y, false);

                self.registers.set_r8(Register8b::A, result);
                4
            }
            0x92 => {
                // SUB A, D
                let a = self.registers.get_r8(Register8b::A);
                let y = self.registers.get_r8(Register8b::D);
                let result = self.alu_sub_bytes(a, y, false);

                self.registers.set_r8(Register8b::A, result);
                4
            }
            0x93 => {
                // SUB A, E
                let a = self.registers.get_r8(Register8b::A);
                let y = self.registers.get_r8(Register8b::E);
                let result = self.alu_sub_bytes(a, y, false);

                self.registers.set_r8(Register8b::A, result);
                4
            }
            0x94 => {
                // SUB A, H
                let a = self.registers.get_r8(Register8b::A);
                let y = self.registers.get_r8(Register8b::H);
                let result = self.alu_sub_bytes(a, y, false);

                self.registers.set_r8(Register8b::A, result);
                4
            }
            0x95 => {
                // SUB A, L
                let a = self.registers.get_r8(Register8b::A);
                let y = self.registers.get_r8(Register8b::L);
                let result = self.alu_sub_bytes(a, y, false);

                self.registers.set_r8(Register8b::A, result);
                4
            }
            0x96 => {
                self.unimpl_instr();
            }
            0x97 => {
                // SUB A, A
                let a = self.registers.get_r8(Register8b::A);
                let result = self.alu_sub_bytes(a, a, false);

                self.registers.set_r8(Register8b::A, result);
                4
            }
            0x98 => {
                // SBC A, B
                let a = self.registers.get_r8(Register8b::A);
                let y = self.registers.get_r8(Register8b::B);
                let result = self.alu_sub_bytes(a, y, true);

                self.registers.set_r8(Register8b::A, result);
                4
            }
            0x99 => {
                // SBC A, C
                let a = self.registers.get_r8(Register8b::A);
                let y = self.registers.get_r8(Register8b::C);
                let result = self.alu_sub_bytes(a, y, true);

                self.registers.set_r8(Register8b::A, result);
                4
            }
            0x9A => {
                // SBC A, D
                let a = self.registers.get_r8(Register8b::A);
                let y = self.registers.get_r8(Register8b::D);
                let result = self.alu_sub_bytes(a, y, true);

                self.registers.set_r8(Register8b::A, result);
                4
            }
            0x9B => {
                // SBC A, E
                let a = self.registers.get_r8(Register8b::A);
                let y = self.registers.get_r8(Register8b::E);
                let result = self.alu_sub_bytes(a, y, true);

                self.registers.set_r8(Register8b::A, result);
                4
            }
            0x9C => {
                // SBC A, H
                let a = self.registers.get_r8(Register8b::A);
                let y = self.registers.get_r8(Register8b::H);
                let result = self.alu_sub_bytes(a, y, true);

                self.registers.set_r8(Register8b::A, result);
                4
            }
            0x9D => {
                // SBC A, L
                let a = self.registers.get_r8(Register8b::A);
                let y = self.registers.get_r8(Register8b::L);
                let result = self.alu_sub_bytes(a, y, true);

                self.registers.set_r8(Register8b::A, result);
                4
            }
            0x9E => {
                // SBC A, (HL)
                self.unimpl_instr();
            }
            0x9F => {
                // SBC A, A
                let a = self.registers.get_r8(Register8b::A);
                let result = self.alu_sub_bytes(a, a, true);

                self.registers.set_r8(Register8b::A, result);
                4
            }
            0xA0 => {
                // AND B
                let y = self.registers.get_r8(Register8b::B);
                self.alu_and_a(y);
                4
            }
            0xA1 => {
                // AND C
                let y = self.registers.get_r8(Register8b::C);
                self.alu_and_a(y);
                4
            }
            0xA2 => {
                // AND D
                let y = self.registers.get_r8(Register8b::D);
                self.alu_and_a(y);
                4
            }
            0xA3 => {
                // AND E
                let y = self.registers.get_r8(Register8b::E);
                self.alu_and_a(y);
                4
            }
            0xA4 => {
                // AND H
                let y = self.registers.get_r8(Register8b::H);
                self.alu_and_a(y);
                4
            }
            0xA5 => {
                // AND L
                let y = self.registers.get_r8(Register8b::L);
                self.alu_and_a(y);
                4
            }
            0xA6 => {
                // AND (HL)
                self.unimpl_instr();
            }
            0xA7 => {
                // AND A
                let y = self.registers.get_r8(Register8b::A);
                self.alu_and_a(y);
                4
            }
            0xA8 => {
                // XOR B
                let y = self.registers.get_r8(Register8b::B);
                self.alu_xor_a(y);
                4
            }
            0xA9 => {
                // XOR C
                let y = self.registers.get_r8(Register8b::C);
                self.alu_xor_a(y);
                4
            }
            0xAA => {
                // XOR D
                let y = self.registers.get_r8(Register8b::D);
                self.alu_xor_a(y);
                4
            }
            0xAB => {
                // XOR E
                let y = self.registers.get_r8(Register8b::E);
                self.alu_xor_a(y);
                4
            }
            0xAC => {
                // XOR H
                let y = self.registers.get_r8(Register8b::H);
                self.alu_xor_a(y);
                4
            }
            0xAD => {
                // XOR L
                let y = self.registers.get_r8(Register8b::L);
                self.alu_xor_a(y);
                4
            }
            0xAE => {
                // AND (HL)
                self.unimpl_instr();
            }
            0xAF => {
                // XOR A
                let y = self.registers.get_r8(Register8b::A);
                self.alu_xor_a(y);
                4
            }
            0xB0 => {
                // OR B
                let y = self.registers.get_r8(Register8b::B);
                self.alu_or_a(y);
                4
            }
            0xB1 => {
                // OR C
                let y = self.registers.get_r8(Register8b::C);
                self.alu_or_a(y);
                4
            }
            0xB2 => {
                // OR D
                let y = self.registers.get_r8(Register8b::D);
                self.alu_or_a(y);
                4
            }
            0xB3 => {
                // OR E
                let y = self.registers.get_r8(Register8b::E);
                self.alu_or_a(y);
                4
            }
            0xB4 => {
                // OR H
                let y = self.registers.get_r8(Register8b::H);
                self.alu_or_a(y);
                4
            }
            0xB5 => {
                // OR L
                let y = self.registers.get_r8(Register8b::L);
                self.alu_or_a(y);
                4
            }
            0xB6 => {
                // OR (HL)
                self.unimpl_instr();
            }
            0xB7 => {
                // OR A
                let y = self.registers.get_r8(Register8b::A);
                self.alu_or_a(y);
                4
            }
            0xB8 => {
                // CP B
                let y = self.registers.get_r8(Register8b::B);
                self.alu_cp_a(y);
                4
            }
            0xB9 => {
                // CP C
                let y = self.registers.get_r8(Register8b::C);
                self.alu_cp_a(y);
                4
            }
            0xBA => {
                // CP D
                let y = self.registers.get_r8(Register8b::D);
                self.alu_cp_a(y);
                4
            }
            0xBB => {
                // CP E
                let y = self.registers.get_r8(Register8b::E);
                self.alu_cp_a(y);
                4
            }
            0xBC => {
                // CP H
                let y = self.registers.get_r8(Register8b::H);
                self.alu_cp_a(y);
                4
            }
            0xBD => {
                // CP L
                let y = self.registers.get_r8(Register8b::L);
                self.alu_cp_a(y);
                4
            }
            0xBE => {
                // CP (HL)
                self.unimpl_instr();
            }
            0xBF => {
                // CP A
                let y = self.registers.get_r8(Register8b::A);
                self.alu_cp_a(y);
                4
            }
            // 0xC0 -> 0xCF
            // 0xD0 -> 0xDF
            // 0xE0 -> 0xEF
            // 0xF0 -> 0xFF
            _ => self.unimpl_instr(),
        }
    }

    fn execute_prefixed_instr(&mut self, instruction: u8) -> u8 {
        self.unimpl_instr();
    }

    /// DEBUG FUNCTION
    ///
    /// placeholder for instructions not yet implemented
    fn unimpl_instr(&self) -> ! {
        unimplemented!("Unimplemented or invalid instruction!")
    }
}

#[cfg(test)]
mod tests;
