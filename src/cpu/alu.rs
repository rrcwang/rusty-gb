use super::*;

impl Cpu {
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
    pub(in crate::cpu) fn alu_add_bytes(&mut self, x: u8, y: u8, use_carry: bool) -> u8 {
        let c: u16 = match use_carry & self.registers.flag_value(Flag::C) {
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
    pub(in crate::cpu) fn alu_sub_bytes(&mut self, x: u8, y: u8, use_carry: bool) -> u8 {
        let c: u16 = match use_carry & self.registers.flag_value(Flag::C) {
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
    pub(in crate::cpu) fn alu_add_words(&mut self, x: u16, y: u16) -> u16 {
        let (x_high, x_low) = word_to_bytes(x);
        let (y_high, y_low) = word_to_bytes(y);

        // need to leave z untouched, but alu_add_bytes modifies Z
        let z = self.registers.flag_value(Flag::Z);

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
    pub(in crate::cpu) fn alu_inc_byte(&mut self, x: u8) -> u8 {
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
    pub(in crate::cpu) fn alu_dec_byte(&mut self, x: u8) -> u8 {
        let result = x.wrapping_sub(1);

        self.registers.set_flag(Flag::Z, result == 0);
        self.registers.set_flag(Flag::N, true);
        self.registers.set_flag(Flag::H, (x & 0x0F) == 0);
        // half-carry condition, borrow from 4->3
        // 0xN0 - 0x01 == 0x(N-1)F <=> 0xN0 & 0x0F == 0
        result
    }

    pub(in crate::cpu) fn alu_and_a(&mut self, y: u8) {
        let a = self.registers.get_r8(Register8b::A);

        let result = a & y;

        self.registers.set_flag(Flag::Z, result == 0);
        self.registers.set_flag(Flag::N, false);
        self.registers.set_flag(Flag::H, true);
        self.registers.set_flag(Flag::C, false);

        self.registers.set_r8(Register8b::A, result);
    }

    pub(in crate::cpu) fn alu_xor_a(&mut self, y: u8) {
        let a = self.registers.get_r8(Register8b::A);

        let result = a ^ y;

        self.registers.set_flag(Flag::Z, result == 0);
        self.registers.set_flag(Flag::N, false);
        self.registers.set_flag(Flag::H, false);
        self.registers.set_flag(Flag::C, false);

        self.registers.set_r8(Register8b::A, result);
    }

    pub(in crate::cpu) fn alu_or_a(&mut self, y: u8) {
        let a = self.registers.get_r8(Register8b::A);

        let result = a | y;

        self.registers.set_flag(Flag::Z, result == 0);
        self.registers.set_flag(Flag::N, false);
        self.registers.set_flag(Flag::H, false);
        self.registers.set_flag(Flag::C, false);

        self.registers.set_r8(Register8b::A, result);
    }

    pub(in crate::cpu) fn alu_cp_a(&mut self, y: u8) {
        let a = self.registers.get_r8(Register8b::A);
        self.alu_sub_bytes(a, y, false);
    }

    pub(in crate::cpu) fn bit_op_rlc(&mut self, x: u8) -> u8 {
        let carry_out = x >> 7;
        let carry_in = self.registers.flag_value(Flag::C) as u8;
        let value = (x << 1) | carry_in;

        self.registers.set_flag(Flag::Z, value == 0);
        self.registers.set_flag(Flag::N, false);
        self.registers.set_flag(Flag::H, false);
        self.registers.set_flag(Flag::C, carry_out == 1);

        value
    }

    pub(in crate::cpu) fn bit_op_rrc(&mut self, x: u8) -> u8 {
        let carry_out = x >> 7;
        let carry_in = self.registers.flag_value(Flag::C) as u8;
        let value = (x << 1) | carry_in;

        self.registers.set_flag(Flag::Z, value == 0);
        self.registers.set_flag(Flag::N, false);
        self.registers.set_flag(Flag::H, false);
        self.registers.set_flag(Flag::C, carry_out == 1);

        value
    }
}
