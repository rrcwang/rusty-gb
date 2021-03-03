use super::*;

// test setup functions
mod common{
    use super::super::*;
    
    /// Tests that the CPU registers are set correctly
    /// 
    /// # Arguments
    /// 
    /// * `cpu`
    /// * `values` - Vector specifying values of flags in order of Flag::{Z, N, H, C} 
    pub fn assert_flags(cpu: Cpu, values: [bool; 4]) {
        assert_eq!(values[0], cpu.registers.get_flag(Flag::Z));
        assert_eq!(values[1], cpu.registers.get_flag(Flag::N));
        assert_eq!(values[2], cpu.registers.get_flag(Flag::H));
        assert_eq!(values[3], cpu.registers.get_flag(Flag::C));
    }
    
}

#[test]
fn cpu_alu_add_bytes_result() {
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
        let result = cpu.alu_add_bytes(*a, *b);
        assert_eq!(a.wrapping_add(*b), result);
    }
    
}

#[test]
fn cpu_alu_add_bytes_flag_zero() {
    let mut cpu = Cpu::new();
    
    let result: u8 = cpu.alu_add_bytes(0x00, 0x00);
    
    assert_eq!(0, result);    
    common::assert_flags(cpu, [true, false, false, false]);
    
}

#[test]
fn cpu_alu_add_bytes_flag_half_carry(){
    
}