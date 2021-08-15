// modules
use crate::{
    memory, registers,
    utils::{bytes_to_word, word_to_bytes},
};
use registers::{Flag, Register16b, Register8b, Registers};

/// Implements arithmetic logic for `Cpu` functions.
mod alu;

/// Implements CPU instructions
mod ops;

/// CPU
pub struct Cpu {
    /// Registers
    registers: Registers,
    /// Halt execution flag
    halted: bool,
    /// Interrupt enable flag
    interrupt_master_enable: bool, // IME
    /// Memory controller
    mmu: memory::Mmu,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            // TODO: what are the initilization values here?
            registers: Registers::new(),
            halted: false,
            interrupt_master_enable: false,
            mmu: memory::Mmu::new(),
        }
    }

    /// Read a byte pointed to by SP and increment the program counter by 1
    pub(in crate::cpu) fn fetch_byte(&mut self) -> u8 {
        let value: u8 = self.mmu.read_byte(self.registers.pc);
        self.registers.pc = self.registers.pc.wrapping_add(1);
        value
    }

    /// Read a word pointed to by SP and increment the program counter by 2
    pub(in crate::cpu) fn fetch_word(&mut self) -> u16 {
        let value: u16 = self.mmu.read_word(self.registers.pc);
        self.registers.pc = self.registers.pc.wrapping_add(2);
        value
    }

    /// Loads the value from one 8-bit register to another. Used for LD R8, R8 instructions.
    pub(in crate::cpu) fn ld_regs_8b(&mut self, reg_to: Register8b, reg_from: Register8b) {
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
        // let instruction: u8 = 0x00;
        let instruction: u8 = self.fetch_byte();

        dbg!(
            "CPU executing 0x{:X} at PC: {}",
            instruction,
            self.registers.pc - 1
        );

        self.execute_instr(instruction)
    }

    /// _DEBUG FUNCTION_. Placeholder for instructions not yet implemented
    ///
    /// TODO: dump state?
    pub(in crate::cpu) fn unimpl_instr(&self) -> ! {
        dbg!("{}", &self.registers);

        unimplemented!("Unimplemented or invalid instruction!")
    }
}

#[cfg(test)]
mod tests;
