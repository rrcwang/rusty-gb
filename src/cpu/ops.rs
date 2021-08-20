use super::*;

impl Cpu {
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
    /// `t_states: u8` - Number of clock ticks taken to run instruction. Doesn't simulate per clock cycle right now, only multiples of 4
    pub(crate) fn execute_instr(&mut self, instruction: u8) -> u8 {
        match instruction {
            // 0x00 -> 0x0F
            0x00 => 4, // NOP   | 0x00          | do nothing for 1 cycle
            0x01 => {
                // LD BC, d16   | 0x01 0xNNNN   | load into BC, 0xNNNN
                let value: u16 = self.fetch_word();
                self.registers.set_r16(Register16b::BC, value);
                12 // program takes 12 T-states
            }
            0x02 => {
                // LD (BC), A   | 0x02          | write byte stored in A to memory location (BC)
                let value = self.registers.get_r8(Register8b::A);
                let address = self.registers.get_r16(Register16b::BC);
                self.mmu.write_byte(address, value);
                8
            }
            0x03 => {
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
                // LD B, d8
                let value: u8 = self.fetch_byte();

                self.registers.set_r8(Register8b::B, value);

                8
            }
            0x07 => {
                // RLCA
                let mut value = self.registers.get_r8(Register8b::A);
                value = self.bit_op_rlc(value);

                self.registers.set_r8(Register8b::A, value);
                self.registers.set_flag(Flag::Z, false);

                4
            }
            0x08 => {
                // LD (a16), SP | 0x03 0xNNNN    | write stack pointer, u16 to memory address in operand
                let address = self.fetch_word();
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
                // LD A, (BC)
                let address = self.registers.get_r16(Register16b::BC);
                let value = self.mmu.read_byte(address);
                self.registers.set_r8(Register8b::A, value);
                8
            }
            0x0B => {
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
                let value = self.fetch_byte();
                self.registers.set_r8(Register8b::C, value);
                8
            }
            0x0F => {
                // RRCA
                self.unimpl_instr();
                4
            }
            0x10 => {
                // STOP
                self.unimpl_instr();
                self.registers.pc = self.registers.pc.wrapping_add(1);
                4
            }
            0x11 => {
                // LD DE, d16
                let value = self.fetch_word();
                self.registers.set_r16(Register16b::DE, value);
                12
            }
            0x12 => {
                // LD (DE), A
                let value = self.registers.get_r8(Register8b::A);
                let address = self.registers.get_r16(Register16b::DE);
                self.mmu.write_byte(address, value);
                8
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
            0x16 => {
                // LD D, d8
                let value: u8 = self.fetch_byte();
                self.registers.set_r8(Register8b::D, value);
                8
            }
            0x19 => {
                // ADD HL, DE
                let hl = self.registers.get_r16(Register16b::HL);
                let de = self.registers.get_r16(Register16b::DE);
                let result = self.alu_add_words(hl, de);
                self.registers.set_r16(Register16b::HL, result);
                8
            }
            0x1A => {
                // LD A, (DE)
                let address = self.registers.get_r16(Register16b::DE);
                let value = self.mmu.read_byte(address);
                self.registers.set_r8(Register8b::A, value);
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
            0x1E => {
                // LD E, d8
                let value = self.fetch_byte();
                self.registers.set_r8(Register8b::E, value);
                8
            }
            // 0x20 -> 0x2F
            0x21 => {
                // LD HL, d16
                let value = self.fetch_word();
                self.registers.set_r16(Register16b::HL, value);
                12
            }
            0x22 => {
                // LD (HL+), A
                let value = self.registers.get_r8(Register8b::A);
                let address = self.registers.get_r16(Register16b::HL);
                self.mmu.write_byte(address, value);

                // increment HL
                self.registers
                    .set_r16(Register16b::HL, address.wrapping_add(1));

                8
            }
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
            0x26 => {
                // LD H, d8
                let value: u8 = self.fetch_byte();
                self.registers.set_r8(Register8b::H, value);
                8
            }
            0x27 => {
                // DAA
                // https://stackoverflow.com/questions/8119577/z80-daa-instruction/8119836
                self.unimpl_instr();
            }
            0x29 => {
                // ADD HL, HL
                let hl = self.registers.get_r16(Register16b::HL);
                let result = self.alu_add_words(hl, hl);
                self.registers.set_r16(Register16b::HL, result);
                8
            }
            0x2A => {
                // LD A, (HL+)
                let address = self.registers.get_r16(Register16b::HL);
                self.registers
                    .set_r16(Register16b::HL, address.wrapping_add(1));
                let value = self.mmu.read_byte(address);
                self.registers.set_r8(Register8b::A, value);
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
            0x2E => {
                // LD L, d8
                let value = self.fetch_byte();
                self.registers.set_r8(Register8b::L, value);
                8
            }
            0x2F => {
                // CPL          | Complement accumulator A
                let a = self.registers.get_r8(Register8b::A);
                self.registers.set_r8(Register8b::A, !a);
                self.registers.set_flag(Flag::N, true);
                self.registers.set_flag(Flag::H, true);
                4
            }
            // 0x30 -> 0x3F
            0x31 => {
                // LD SP, d16
                let value = self.fetch_word();
                self.registers.set_r16(Register16b::SP, value);
                12
            }
            0x32 => {
                // LD (HL-), A
                let value = self.registers.get_r8(Register8b::A);
                let address = self.registers.get_r16(Register16b::HL);
                self.mmu.write_byte(address, value);

                // decrement HL
                self.registers
                    .set_r16(Register16b::HL, address.wrapping_sub(1));

                8
            }
            0x33 => {
                // INC SP
                self.registers.sp = self.registers.sp.wrapping_add(1);
                8
            }
            0x34 => {
                // INC (HL)
                let address = self.registers.get_r16(Register16b::HL);
                let value = self.mmu.read_byte(address);
                let value = self.alu_inc_byte(value);
                self.mmu.write_byte(address, value);
                12
            }
            0x35 => {
                // DEC (HL)
                let address = self.registers.get_r16(Register16b::HL);
                let value = self.mmu.read_byte(address);
                let value = self.alu_dec_byte(value);
                self.mmu.write_byte(address, value);
                12
            }
            0x36 => {
                // LD (HL), d8
                let value: u8 = self.fetch_byte();
                let address = self.registers.get_r16(Register16b::HL);
                self.mmu.write_byte(address, value);
                12
            }
            0x37 => {
                // SCF
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, false);
                self.registers.set_flag(Flag::C, true);
                4
            }
            0x38 => {
                // SCF
                // maybe should be done with set_flag for clarity?
                let mut flags = self.registers.get_r8(Register8b::F);
                flags |= 0b_0001_0000; // set bit 4
                flags &= 0b_1001_1111; // clear bits 5, 6
                self.registers.set_r8(Register8b::F, flags);
                4
            }
            0x39 => {
                // ADD HL, SP
                let hl = self.registers.get_r16(Register16b::HL);
                let sp = self.registers.get_r16(Register16b::SP);
                let result = self.alu_add_words(hl, sp);
                self.registers.set_r16(Register16b::HL, result);
                8
            }
            0x3A => {
                // LD A, (HL-)
                let address = self.registers.get_r16(Register16b::HL);
                self.registers
                    .set_r16(Register16b::HL, address.wrapping_sub(1));
                let value = self.mmu.read_byte(address);
                self.registers.set_r8(Register8b::A, value);
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
            0x3E => {
                // LD E, d8
                let value = self.fetch_byte();
                self.registers.set_r8(Register8b::A, value);
                8
            }
            0x3F => {
                // CCF        | complement C flag
                let c = self.registers.flag_value(Flag::C);
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, false);
                self.registers.set_flag(Flag::C, !c);
                4
            }
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
                // LD B, (HL)
                let address = self.registers.get_r16(Register16b::HL);
                let value = self.mmu.read_byte(address);
                self.registers.set_r8(Register8b::B, value);
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
                // LD C, (HL)
                let address = self.registers.get_r16(Register16b::HL);
                let value = self.mmu.read_byte(address);
                self.registers.set_r8(Register8b::C, value);
                8
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
                // LD D, (HL)
                let address = self.registers.get_r16(Register16b::HL);
                let value = self.mmu.read_byte(address);
                self.registers.set_r8(Register8b::D, value);
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
                // LD E, (HL)
                let address = self.registers.get_r16(Register16b::HL);
                let value = self.mmu.read_byte(address);
                self.registers.set_r8(Register8b::E, value);
                8
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
                // LD H, (HL)
                let address = self.registers.get_r16(Register16b::HL);
                let value = self.mmu.read_byte(address);
                self.registers.set_r8(Register8b::H, value);
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
                // LD L, (HL)
                let address = self.registers.get_r16(Register16b::HL);
                let value = self.mmu.read_byte(address);
                self.registers.set_r8(Register8b::L, value);
                8
            }
            0x6F => {
                // LD L, A
                self.ld_regs_8b(Register8b::L, Register8b::A);
                4
            }
            0x70 => {
                // LD (HL), B
                let address = self.registers.get_r16(Register16b::HL);
                let value = self.registers.get_r8(Register8b::B);
                self.mmu.write_byte(address, value);
                8
            }
            0x71 => {
                // LD (HL), C
                let address = self.registers.get_r16(Register16b::HL);
                let value = self.registers.get_r8(Register8b::C);
                self.mmu.write_byte(address, value);
                8
            }
            0x72 => {
                // LD (HL), D
                let address = self.registers.get_r16(Register16b::HL);
                let value = self.registers.get_r8(Register8b::D);
                self.mmu.write_byte(address, value);
                8
            }
            0x73 => {
                // LD (HL), E
                let address = self.registers.get_r16(Register16b::HL);
                let value = self.registers.get_r8(Register8b::E);
                self.mmu.write_byte(address, value);
                8
            }
            0x74 => {
                // LD (HL), H
                let address = self.registers.get_r16(Register16b::HL);
                let value = self.registers.get_r8(Register8b::H);
                self.mmu.write_byte(address, value);
                8
            }
            0x75 => {
                // LD (HL), L
                let address = self.registers.get_r16(Register16b::HL);
                let value = self.registers.get_r8(Register8b::L);
                self.mmu.write_byte(address, value);
                8
            }
            0x76 => {
                // HALT
                self.unimpl_instr();
                4
            }
            0x77 => {
                // LD (HL), A
                let address = self.registers.get_r16(Register16b::HL);
                let value = self.registers.get_r8(Register8b::A);
                self.mmu.write_byte(address, value);
                8
            }
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
                // LD A, (HL)
                let address = self.registers.get_r16(Register16b::HL);
                let value = self.mmu.read_byte(address);
                self.registers.set_r8(Register8b::A, value);
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
                // ADD A, (HL)
                let a = self.registers.get_r8(Register8b::A);
                let address = self.registers.get_r16(Register16b::HL);
                let y = self.mmu.read_byte(address);
                let result = self.alu_add_bytes(a, y, false);

                self.registers.set_r8(Register8b::A, result);
                8
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
                let a = self.registers.get_r8(Register8b::A);
                let address = self.registers.get_r16(Register16b::HL);
                let y = self.mmu.read_byte(address);
                let result = self.alu_add_bytes(a, y, true);

                self.registers.set_r8(Register8b::A, result);
                8
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
                // SUB A, (HL)
                let a = self.registers.get_r8(Register8b::A);
                let address = self.registers.get_r16(Register16b::HL);
                let y = self.mmu.read_byte(address);
                let result = self.alu_sub_bytes(a, y, false);

                self.registers.set_r8(Register8b::A, result);
                8
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
                let a = self.registers.get_r8(Register8b::A);
                let address = self.registers.get_r16(Register16b::HL);
                let y = self.mmu.read_byte(address);
                let result = self.alu_sub_bytes(a, y, true);

                self.registers.set_r8(Register8b::A, result);
                8
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
                let address = self.registers.get_r16(Register16b::HL);
                let y = self.mmu.read_byte(address);
                self.alu_and_a(y);
                8
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
                // XOR (HL)
                let address = self.registers.get_r16(Register16b::HL);
                let y = self.mmu.read_byte(address);
                self.alu_xor_a(y);
                8
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
                let address = self.registers.get_r16(Register16b::HL);
                let y = self.mmu.read_byte(address);
                self.alu_or_a(y);
                8
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
                let address = self.registers.get_r16(Register16b::HL);
                let y = self.mmu.read_byte(address);
                self.alu_cp_a(y);
                8
            }
            0xBF => {
                // CP A
                let y = self.registers.get_r8(Register8b::A);
                self.alu_cp_a(y);
                4
            }
            // 0xC0 -> 0xCF
            0xC1 => {
                // POP BC
                let value = self.mmu.read_word(self.registers.sp);
                self.registers.sp = self.registers.sp.wrapping_add(2);
                self.registers.set_r16(Register16b::BC, value);
                12
            }
            0xC5 => {
                // PUSH BC
                let value = self.registers.get_r16(Register16b::BC);
                self.registers.sp = self.registers.sp.wrapping_sub(2);
                self.mmu.write_word(self.registers.sp, value);
                16
            }
            0xC6 => {
                // ADD A, d8
                let y = self.fetch_byte();
                let a = self.registers.get_r8(Register8b::A);
                self.alu_add_bytes(a, y, false);
                8
            }
            0xCB => {
                // PREFIX
                let instr = self.fetch_byte();
                self.execute_prefixed_instr(instr)
            }
            0xCE => {
                // ADC A, d8
                let y = self.fetch_byte();
                let a = self.registers.get_r8(Register8b::A);
                self.alu_add_bytes(a, y, true);
                8
            }
            // 0xD0 -> 0xDF
            0xD1 => {
                // POP DE
                let value = self.mmu.read_word(self.registers.sp);
                self.registers.sp = self.registers.sp.wrapping_add(2);
                self.registers.set_r16(Register16b::DE, value);
                12
            }
            0xD5 => {
                // PUSH DE
                let value = self.registers.get_r16(Register16b::DE);
                self.registers.sp = self.registers.sp.wrapping_sub(2);
                self.mmu.write_word(self.registers.sp, value);
                16
            }
            0xD6 => {
                // SUB A, d8
                let y = self.fetch_byte();
                let a = self.registers.get_r8(Register8b::A);
                self.alu_sub_bytes(a, y, false);
                8
            }
            0xDE => {
                // SBC A, d8
                let y = self.fetch_byte();
                let a = self.registers.get_r8(Register8b::A);
                self.alu_sub_bytes(a, y, true);
                8
            }
            // 0xE0 -> 0xEF
            0xE0 => {
                // LDH (a8), A
                let address = self.fetch_byte() as u16 + 0xFF00;
                assert!(0xFF00 <= address); // DEBUG
                let value = self.registers.get_r8(Register8b::A);
                self.mmu.write_byte(address, value);
                12
            }
            0xE1 => {
                // POP HL
                let value = self.mmu.read_word(self.registers.sp);
                self.registers.sp = self.registers.sp.wrapping_add(2);
                self.registers.set_r16(Register16b::HL, value);
                12
            }
            0xE2 => {
                // LD (C), A
                let address = self.registers.get_r8(Register8b::C) as u16 + 0xFF00;
                assert!(0xFF00 <= address); // DEBUG
                let value = self.registers.get_r8(Register8b::A);
                self.mmu.write_byte(address, value);
                12
            }
            0xE5 => {
                // PUSH HL
                let value = self.registers.get_r16(Register16b::HL);
                self.registers.sp = self.registers.sp.wrapping_sub(2);
                self.mmu.write_word(self.registers.sp, value);
                16
            }
            0xE6 => {
                // AND d8
                let y = self.fetch_byte();
                self.alu_and_a(y);
                8
            }
            0xE8 => {
                // ADD SP, r8
                // two's complement should work here
                let offset = self.fetch_byte() as i8 as u16;
                // (255 as i8) as u16 => (-1) as u16 => 25565
                self.registers.sp = self.alu_add_words(self.registers.sp, offset);
                self.registers.set_flag(Flag::Z, false);
                16
            }
            0xEA => {
                // LD (a16), A
                let address = self.fetch_word();
                let value = self.registers.get_r8(Register8b::A);
                self.mmu.write_byte(address, value);
                16
            }
            0xEE => {
                // XOR d8
                let y = self.fetch_byte();
                self.alu_xor_a(y);
                8
            }
            0xEF => {
                // RST 28h
                16
            }
            // 0xF0 -> 0xFF
            0xF0 => {
                // LDH A, (a8)
                let address = self.fetch_byte() as u16 + 0xFF00;
                assert!(0xFF00 <= address); // DEBUG
                let value = self.mmu.read_byte(address);
                self.registers.set_r8(Register8b::A, value);
                12
            }
            0xF1 => {
                // POP AF
                let value = self.mmu.read_word(self.registers.sp);
                self.registers.sp = self.registers.sp.wrapping_add(2);
                self.registers.set_r16(Register16b::AF, value);
                12
            }
            0xF2 => {
                // LD A, (C)
                let address = self.registers.get_r8(Register8b::C) as u16 + 0xFF00;
                assert!(0xFF00 <= address); // DEBUG
                let value = self.mmu.read_byte(address);
                self.registers.set_r8(Register8b::A, value);
                12
            }
            0xF3 => {
                // DI       | disable interrupts
                self.interrupt_master_enable = false;
                4
            }
            0xF5 => {
                // PUSH AF
                let value = self.registers.get_r16(Register16b::AF) | 0xFFF0; // must clear empty flag bits?
                self.registers.sp = self.registers.sp.wrapping_sub(2);
                self.mmu.write_word(self.registers.sp, value);
                16
            }
            0xF6 => {
                // OR d8
                let y = self.fetch_byte();
                self.alu_or_a(y);
                8
            }
            0xF8 => {
                // LD HL, SP + r8
                let offset = self.fetch_byte() as i8 as u16; // see 0xE8
                let value = self.alu_add_words(self.registers.sp, offset);
                self.registers.set_r16(Register16b::HL, value);
                self.registers.set_flag(Flag::Z, false);
                12
            }
            0xFA => {
                // LD A, (a16)
                let address = self.fetch_word();
                let value = self.mmu.read_byte(address);
                self.registers.set_r8(Register8b::A, value);
                16
            }
            0xFB => {
                // EI       | enable interrupts
                self.interrupt_master_enable = true;
                4
            }
            0xF9 => {
                // LD SP, HL
                let value = self.registers.get_r16(Register16b::HL);
                self.registers.set_r16(Register16b::SP, value);
                8
            }
            0xFE => {
                // CP d8
                let y = self.fetch_byte();
                self.alu_cp_a(y);
                8
            }
            0xFF => {
                // RST 38h
                self.unimpl_instr();
                16
            }
            _ => self.unimpl_instr(),
        }
    }

    fn execute_prefixed_instr(&mut self, instruction: u8) -> u8 {
        self.unimpl_instr();

        // interpret target r8 or (HL)

        // read target and store in value

        // interpret for desired instruction

        // modify value

        // store back into target
    }
}
