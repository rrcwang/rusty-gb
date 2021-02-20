
mod register;

#[cfg(test)]
mod register_tests {

    #[test]
    fn register_init_zero() {
        let register::Registers::registers_8b_str = vec!["a", "f", "b", "c", "d", "e", "h", "l"];

        let registers = register::Registers::new();

        for reg in registers_8b_str {
            assert_eq!(0, register::Registers::registers.get_8b_reg(reg));
        }
    }
}

/*
.
|-- lib.rs
|   |-- register_tests
|       |-- register_init_zero()
|-- main.rs
|-- register.rs

mod register;
*/