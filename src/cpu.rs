use crate::memory;
use crate::registers;
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

// TODO:
// ALU
//  * add carry flag for ALU add, sub           -- DONE, tested
//  * write tests for ALU sub                   -- DONE, tested
//  * refactor?? ALU to separate file?
// Instructions
//  Data
//      * LD A, R8
//      * LD with pointers
//      * INC R8, INC R16
//          ...
//  * Arithmetic
//      * ADD, ADC  R8
//      * SUB, SBC  R8
//          ...
//  * Logical
//      * AND
//      * XOR
//      * OR
//      * CP - comparison with A
//          ... flag, A
//  * Misc / control
//      * NOP -- DONE
//          ...
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

    /// Computes (x - y) and sets the // TODO: use carry flag for SBC instructions!! Subtract w/ carry or borrow?
    ///
    ///
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

    /// Loads the value from one 8-bit register to another. Used for LD R8, R8 instructions.
    fn ld_regs_8b(&mut self, reg_to: Register8b, reg_from: Register8b) {
        /* if reg_from == reg_to { // ignore for now, assuming is not called for same reg instructions
            return
        } */
        let value = self.registers.get_8b_reg(reg_from);
        self.registers.set_8b_reg(reg_to, value);
    }

    /// returns the duration taken by the instruction in clock ticks taken
    /// one CPU cycle == "M-cycle"
    ///     => four clock ticks == four "T-states"
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
        //      * do thing
        //      * set flags
        //      * increment program counter past immediate (operands) if they exist
        //      * return number of cycles
        //  op-codes documented here: https://gbdev.io/gb-opcodes/optables/
        match instruction {
            // 0x00 -> 0x0F
            0x00 => 4, // NOP   | 0x00          | do nothing for 1 cycle
            0x01 => {
                // LD BC, d16   | 0x01 0xIIII   | load into BC, 0xIIII
                let value: u16 = self.mmu.read_word(self.registers.pc); // read data from program
                self.registers.pc += 2; // length of operands

                self.registers.set_16b_reg(Register16b::BC, value);

                12 // program takes 12 T-states
            }
            0x02 => {
                // LD (BC), A   | 0x02          | load byte stored at memory location pointed to by BC into A
                let value: u8 = self
                    .mmu
                    .read_byte(self.registers.get_16b_reg(Register16b::BC));

                self.registers.set_8b_reg(Register8b::A, value);

                8
            }
            0x03 => {
                // INC BC      | 0x03            | increment BC register by 1
                // TODO: implement ALU and addition behaviour
                self.unimpl_instr();

                8
            }
            0x04 => {
                // INC B
                self.unimpl_instr();

                4
            }
            0x05 => {
                // DEC B
                self.unimpl_instr();

                4
            }
            0x06 => {
                // LD B, d8
                let value: u8 = self.mmu.read_byte(self.registers.pc);
                self.registers.pc += 1;

                self.registers.set_8b_reg(Register8b::B, value);

                8
            }
            // TODO: opcodes 0x07 -> 0x3F

            // 0x10 -> 0x1F
            // 0x20 -> 0x2F
            // 0x30 -> 0x3F

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
                // LD D, B    | does nothing
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
                self.unimpl_instr();

                self.registers.set_flag(Flag::N, false); // subtraction flag
                4
            }
            // 0x90 -> 0x9F
            // 0xA0 -> 0xAF
            // 0xB0 -> 0xBF
            // 0xC0 -> 0xCF
            // 0xD0 -> 0xDF
            // 0xE0 -> 0xEF
            // 0xF0 -> 0xFF
            _ => self.unimpl_instr(),
        }
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
