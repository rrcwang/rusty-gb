use super::*;

impl Cpu {
    /// Adds two byte length values and sets the appropriate flags in the F register for CPU instructions.
    ///
    /// Applicable for instructions 0x80..0x8F
    ///
    /// # Arguments
    /// `x`, `y` - Operands for the addition
    ///
    /// # Flags
    /// `Z` if result == 0
    /// `N = 0`
    /// `H` if sum of lower 4 bits overflows
    /// `C` if sum of 8 bits overflows
    pub(in crate::cpu) fn alu_add_bytes(&mut self, x: u8, y: u8, use_carry: bool) -> u8 {
        let c: u16 = match use_carry & self.registers.flag_value(Flag::C) {
            true => 1,
            false => 0,
        };
        let x = x as u16;
        let y = (y as u16).wrapping_add(c);

        let result = x.wrapping_add(y);

        self.registers.set_flag(Flag::Z, result & 0x00FF == 0);
        self.registers.set_flag(Flag::N, false);
        // half-carry: https://stackoverflow.com/questions/62006764/how-is-xor-applied-when-determining-carry
        //  x + y == x ^ y ^ carry_bits
        //  x ^ y ^ sum == carry_bits
        //  (x ^ y ^ sum) & 0x10 == carry_bits & 0x10 == carry out for bit 4
        self.registers
            .set_flag(Flag::H, ((x ^ y ^ result) & 0x10) != 0);
        self.registers.set_flag(Flag::C, (result & 0x100) != 0);

        result as u8 // return 8-bit
    }

    /// Computes `x - y` and sets the relevant flags.
    ///
    /// # Arguments
    /// `x`, `y` - Operands for the subtraction
    ///
    /// # Flags
    /// `Z` if result == 0
    /// `N = 1`
    /// `H` if borrow from bit 4
    /// `C` if borrow from bit 8
    pub(in crate::cpu) fn alu_sub_bytes(&mut self, x: u8, y: u8, use_carry: bool) -> u8 {
        let c: u16 = (use_carry & self.registers.flag_value(Flag::C)) as u16;
        let x = x as u16;
        let y = (y as u16).wrapping_add(c);

        let result = x.wrapping_sub(y);

        self.registers.set_flag(Flag::Z, result & 0x00FF == 0);
        self.registers.set_flag(Flag::N, true);
        self.registers.set_flag(Flag::H, (y & 0x0F) > (x & 0x0F)); // x ^ (!y) ^ result TODO: check potential off-by-one?
        self.registers.set_flag(Flag::C, (result & 0x100) != 0); // should work due to two's complement subtraction? TODO: verify

        result as u8
    }

    /// Adds two unsigned words, and sets the appropriate flags.
    ///
    /// Implemented as chaining two byte length adds together.
    /// Used in ADD HL, r16 instructions.
    ///
    /// # Arguments
    /// * `(x, y)` - `u16`s
    ///
    /// # Output
    /// * Result of `x + y`
    ///
    /// # Flags
    /// * `N = 0`
    /// * `H` if bit 11 overflows  TODO: verify this
    /// * `C` if bit 15 overflows
    pub(in crate::cpu) fn alu_add_words(&mut self, x: u16, y: u16) -> u16 {
        let (x_high, x_low) = word_to_bytes(x);
        let (y_high, y_low) = word_to_bytes(y);

        let z = self.registers.flag_value(Flag::Z); // cache value of `Z` here to avoid modification

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

    /// Computes the value of `y AND val(Flag::A)` and stores the result in `A`. `A = A AND y`.
    ///
    /// # Argument
    /// * `y` - `u8` operand
    ///
    /// # Flags
    /// * `Z` if result is 0
    /// * `N = 0`
    /// * `H = 1`
    /// * `C = 0`
    pub(in crate::cpu) fn alu_and_a(&mut self, y: u8) {
        let a = self.registers.get_r8(Register8b::A);

        let result = a & y;

        self.registers.set_flag(Flag::Z, result == 0);
        self.registers.set_flag(Flag::N, false);
        self.registers.set_flag(Flag::H, true);
        self.registers.set_flag(Flag::C, false);

        self.registers.set_r8(Register8b::A, result);
    }

    /// Computes the value of `y XOR val(Flag::A)` and stores the result in `A`. `A = A XOR y`.
    ///
    /// # Argument
    /// * `y` - `u8` operand
    ///
    /// # Flags
    /// * `Z` if result is 0
    /// * `N = 0`
    /// * `H = 1`
    /// * `C = 0`
    pub(in crate::cpu) fn alu_xor_a(&mut self, y: u8) {
        let a = self.registers.get_r8(Register8b::A);

        let result = a ^ y;

        self.registers.set_flag(Flag::Z, result == 0);
        self.registers.set_flag(Flag::N, false);
        self.registers.set_flag(Flag::H, false);
        self.registers.set_flag(Flag::C, false);

        self.registers.set_r8(Register8b::A, result);
    }

    /// Computes the value of `y OR val(Flag::A)` and stores the result in `A`. `A = A OR y`
    ///
    /// # Argument
    /// * `y` - `u8` operand
    ///
    /// # Flags
    /// * `Z` if result is 0
    /// * `N = 0`
    /// * `H = 0`
    /// * `C = 0`
    pub(in crate::cpu) fn alu_or_a(&mut self, y: u8) {
        let a = self.registers.get_r8(Register8b::A);

        let result = a | y;

        self.registers.set_flag(Flag::Z, result == 0);
        self.registers.set_flag(Flag::N, false);
        self.registers.set_flag(Flag::H, false);
        self.registers.set_flag(Flag::C, false);

        self.registers.set_r8(Register8b::A, result);
    }

    /// Compares the value of the operand `y` with the value in `A`. This is done by calling `SUB A, y` but not storing the result in `A`.
    ///
    /// # Argument
    /// * `y` - `u8` operand
    ///
    /// # Flags
    /// * `Z` if result == 0, iff`A == y`
    /// * `N = 1`
    /// * `H` if borrow from bit 4
    /// * `C` if borrow from bit 8, iff `A < y`
    pub(in crate::cpu) fn alu_cp_a(&mut self, y: u8) {
        let a = self.registers.get_r8(Register8b::A);
        let _ = self.alu_sub_bytes(a, y, false);
    }

    /// "Rotates" or shifts the value of `x` to the left by 1 bit. The flag `C` is "rotated in".
    ///
    /// # Argument
    /// * `x` - `u8` operand
    ///
    /// # Flags
    /// * `Z` if result == 0
    /// * `N = 0`
    /// * `H = 0`
    /// * `C` the value of the most significant bit in `x`, which is shifted out.
    pub(in crate::cpu) fn bit_op_rlc(&mut self, x: u8) -> u8 {
        let carry_out = x >> 7; // fills with 0
        let carry_in = self.registers.flag_value(Flag::C) as u8;
        let value = (x << 1) | carry_in;

        self.registers.set_flag(Flag::Z, value == 0);
        self.registers.set_flag(Flag::N, false);
        self.registers.set_flag(Flag::H, false);
        self.registers.set_flag(Flag::C, carry_out != 0);

        value
    }

    /// "Rotates" or shifts the value of `x` to the right by 1 bit. The flag `C` is "rotated in".
    ///
    /// # Argument
    /// * `x` - `u8` operand
    ///
    /// # Flags
    /// * `Z` if result == 0
    /// * `N = 0`
    /// * `H = 0`
    /// * `C` the value of the least significant bit in `x`, which is shifted out.
    pub(in crate::cpu) fn bit_op_rrc(&mut self, x: u8) -> u8 {
        let carry_out = x << 7; // fills with 0
        let carry_in = self.registers.flag_value(Flag::C) as u8;
        let value = (x >> 1) | carry_in;

        self.registers.set_flag(Flag::Z, value == 0);
        self.registers.set_flag(Flag::N, false);
        self.registers.set_flag(Flag::H, false);
        self.registers.set_flag(Flag::C, carry_out != 0);

        value
    }
}
