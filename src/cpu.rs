use crate::registers;
use registers::Register16b;
use registers::Register8b;
use registers::Flag;
use crate::memory;

pub struct Cpu {
    registers: registers::Registers,
    // CPU counters/states
    halted: bool,
    interrupt_master_enable: bool,  // IME
    // memory
    mmu: memory::MMU,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu { // TODO: what are the initilization values here?
            registers: registers::Registers::new(),
            halted: false,
            interrupt_master_enable: false, 
            mmu: memory::MMU::new(),
        }
    }
    
    /// Adds two byte length values and sets the relevant flags as appropriate for CPU instructions
    /// 
    /// # Flags
    /// Z: true iff result == 0
    /// N: false, not subtraction/negative
    /// H: if sum of lower 4 bits overflows
    /// C: if sum of 8 bits overflows
    fn alu_add_bytes(&mut self, a: u8, b: u8) -> u8 {
        let a = a as u16;
        let b = b as u16;
        let result = (a as u16).wrapping_add(b as u16);
        
        // set flags
        self.registers.set_flag(Flag::N, false); // subtraction
        // set half-carry: https://www.reddit.com/r/EmuDev/comments/692n59/gb_questions_about_halfcarry_and_best/
        self.registers.set_flag(Flag::H, ((a ^ b ^ result) & 0x10) != 0);
        // set full carry
        self.registers.set_flag(Flag::C, (result & 0x100) != 0);
        // cast to 8-bit
        let result = result as u8;
        // set zero flag
        self.registers.set_flag(Flag::Z, result == 0);
        
        result  // return 8-bit 
    }
    

    /// returns the duration taken by the instruction in clock ticks taken
    /// one CPU cycle == "M-cycle"
    ///     => four clock ticks == four "T-states"
    pub fn fetch_and_execute(&mut self) -> u8 {
        // TODO: fetch OP code from ROM
        // 0. fetch next instruction, pointed to by PC
        let instruction: u8 = 0x00;
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
            0x00 => { 4 },   // NOP     | 0x00          | do nothing for 1 cycle
            0x01 => { // LD BC, d16     | 0x01 0xIIII   | load into BC, 0xIIII
                let value: u16 = self.mmu.read_word(self.registers.pc); // read data from program
                self.registers.pc += 2; // length of operands
                
                self.registers.set_16b_reg(Register16b::BC, value);

                12  // program takes 12 T-stattes
            },
            0x02 => { // LD (BC), A     | 0x02          | load byte stored at memory location pointed to by BC into A
                let value: u8 = self.mmu.read_byte(self.registers.get_16b_reg(Register16b::BC));

                self.registers.set_8b_reg(Register8b::A, value);
                
                8
            }
            0x03 => { // INC BC
                // TODO: implement ALU and addition behaviour
                self.unimpl_instr();

                8
            }
            0x04 => { // INC B 
                self.unimpl_instr();

                4
            }
            0x05 => { // DEC B
                self.unimpl_instr();

                4
            }
            0x06 => { // LD B, d8
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
            0x40 => { // LD B, B    | does nothing
                4
            }
            0x41 => { // LD B, C
                let value = self.registers.get_8b_reg(Register8b::C);

                self.registers.set_8b_reg(Register8b::B, value);

                4
            }
            0x42 => { // LD B, D
                let value = self.registers.get_8b_reg(Register8b::D);

                self.registers.set_8b_reg(Register8b::B, value);
                
                4
            }
            0x43 => { // LD B, E
                let value = self.registers.get_8b_reg(Register8b::E);

                self.registers.set_8b_reg(Register8b::B, value);
                
                4
            }
            0x44 => { // LD B, H
                let value = self.registers.get_8b_reg(Register8b::H);

                self.registers.set_8b_reg(Register8b::B, value);
                
                4
            }
            0x45 => { // LD B, L
                let value = self.registers.get_8b_reg(Register8b::L);

                self.registers.set_8b_reg(Register8b::B, value);

                4
            }
            0x46 => { // LD B, (HL)
                self.unimpl_instr();

                8
            }
            0x47 => { // LD B, A                
                let value = self.registers.get_8b_reg(Register8b::A);

                self.registers.set_8b_reg(Register8b::B, value);

                4
            }
            // 0x50 -> 0x5F
            // 0x60 -> 0x6F
            // 0x70 -> 0x7F
            // 0x80 -> 0x8F
            0x80 => { // ADD A, B
                
                
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

            _ => { self.unimpl_instr() },
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
mod test;