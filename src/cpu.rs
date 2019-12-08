use crate::registers::Registers;
use crate::registers::Flags;
use crate::memory::{Memory, N_BYTES};
use crate::op_code::OpCode;
use std::fmt::Debug;
use std::num::Wrapping;

#[derive(Debug)]
struct Cpu {
    stack_pointer: u32,
    program_counter: u16,
    registers: Registers,
    flags: Flags,
    memory: Memory,
}

impl Cpu {
    pub fn new(memory: Memory) -> Cpu {
        Cpu {
            stack_pointer: 0,
            program_counter: 0,
            registers: Registers::new(),
            flags: Flags::new(),
            memory,
        }
    }

    pub fn emulate(&mut self) {
        while (self.program_counter as usize) < self.memory.instructions_len() {
            let op_code: OpCode = self.memory.fetch_byte_at_offset(self.program_counter).into();
            self.execute(&op_code);
        }
    }

    fn execute(&mut self, op_code: &OpCode) {
        match op_code.value {
            0x00 => println!("NOP code"),
            0x37 => self.flags.carry = true,
            0x76 => self.halt(),
            0x3f => self.flags.carry = !self.flags.carry,
            0x40..=0x7f => self.transfer(op_code),
            0x80..=0xbf => self.arithmetic_operation(op_code),
            _ => panic!("Unknown op code")
        }
    }

    fn transfer(&mut self, op_code: &OpCode) {
        let source = self.extract_source_value(op_code);
        let encoded_dest = op_code.extract_first_operand();
        if encoded_dest == 0b110 {
            self.memory.set_byte_at_offset(self.registers.get_hl(), source);
        } else {
            *self.extract_register_address(encoded_dest) = source;
        }
        self.program_counter += 1;
    }

    fn extract_source_value(&mut self, op_code: &OpCode) -> u8 {
        let encoded_source = op_code.extract_second_operand();
        println!("source register: {:#b}", encoded_source);
        return self.extract_memory_or_register(encoded_source);
    }

    fn extract_memory_or_register(&mut self, encoded_source: u8) -> u8 {
        if encoded_source == 0b110 {
            return self.memory.fetch_byte_at_offset(self.registers.get_hl())
        }
        return *self.extract_register_address(encoded_source)
    }

    fn extract_register_address(&mut self, encoded: u8) -> &mut u8 {
        match encoded {
            0b0 => &mut self.registers.b,
            0b001 => &mut self.registers.c,
            0b010 => &mut self.registers.d,
            0b011 => &mut self.registers.e,
            0b100 => &mut self.registers.h,
            0b101 => &mut self.registers.l,
            0b111 => &mut self.registers.acc,
            _ => panic!("Unknown address")
        }
    }

    fn halt(&self) {

    }

    fn arithmetic_operation(&mut self, op_code: &OpCode) {
        let encoded_operation = op_code.extract_first_operand();
        let encoded_addend = op_code.extract_second_operand();
        let value = self.extract_memory_or_register(encoded_addend);
        match encoded_operation {
            0b000 => self.add(value, false),
            0b001 => self.add(value, self.flags.carry),
            0b010 => self.subtract(value, false),
            0b011 => self.subtract(value, self.flags.carry),
            0b100 => self.and(value),
            0b101 => self.xor(value),
            0b110 => self.or(value),
            _ => println!("Could not decode arithmetic operation")
        }
        self.program_counter += 1;
    }

    fn add(&mut self, value: u8, carry: bool) {
        let result: u16 = (self.registers.acc as u16) + (value as u16) + (carry as u16);
        self.flags.set_all(result);
        self.registers.acc = result as u8;
    }

    fn subtract(&mut self, value: u8, carry: bool) {
        let result = (Wrapping(self.registers.acc as u16) - Wrapping(value as u16 + carry as u16)).0;
        self.flags.set_all(result);
        self.registers.acc = result as u8;
    }

    fn and(&mut self, value: u8) {
        let result = self.registers.acc as u16 & value as u16;
        self.flags.set_all(result);
        self.registers.acc = result as u8;
    }

    fn or(&mut self, value: u8) {
        let result = self.registers.acc as u16 | value as u16;
        self.flags.set_all(result);
        self.registers.acc = result as u8;
    }

    fn xor(&mut self, value: u8) {
        let result = self.registers.acc as u16 ^ value as u16;
        self.flags.set_all(result);
        self.registers.acc = result as u8;
    }

    fn comparison(&self, op_code: &OpCode) {

    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::u8;

    fn create_test_cpu(input: Vec<u8>) -> Cpu {
        let mut memory = Memory::new(input);
        Cpu::new(memory)
    }

    #[test]
    fn test_transfer_register() {
        let mut cpu = create_test_cpu(vec![0x50]);
        let result = 8;
        cpu.registers.d = 12;
        cpu.registers.b = result;
        cpu.emulate();
        assert_eq!(cpu.registers.d, result)
    }

    #[test]
    fn test_transfer_same_register() {
        let mut cpu = create_test_cpu(vec![0x49]);
        let result = 8;
        cpu.registers.c = result;
        cpu.emulate();
        assert_eq!(cpu.registers.c, result);
        assert_ne!(cpu.registers.b, result)
    }

    #[test]
    #[should_panic]
    fn test_transfer_memory() {
        let result = 15;
        let mut cpu = create_test_cpu(vec![0x66, result]);
        cpu.registers.h = 0;
        cpu.registers.b = 2;
//        panic::catch_unwind(|| {
        cpu.emulate();
//        });
        assert_eq!(cpu.registers.h, result)
    }

    #[test]
    fn test_add() {
        let mut cpu = create_test_cpu(vec![0x81]);
        cpu.registers.acc = 8;
        cpu.registers.c = 12;
        cpu.emulate();
        assert_eq!(cpu.registers.acc, 20)
    }

    #[test]
    fn test_add_overflow() {
        let mut cpu = create_test_cpu(vec![0x81]);
        cpu.registers.acc = u8::max_value();
        cpu.registers.c = 1;
        cpu.emulate();
        assert_eq!(cpu.registers.acc, 0);
        assert_eq!(cpu.flags.carry, true)
    }

    #[test]
    fn test_add_with_carry() {
        let mut cpu = create_test_cpu(vec![0x8a]);
        cpu.registers.acc = 8;
        cpu.registers.d = 12;
        cpu.flags.carry = true;
        cpu.emulate();
        assert_eq!(cpu.registers.acc, 21)
    }

    #[test]
    fn test_set_zero_flag() {
        let mut cpu = create_test_cpu(vec![0x83]);
        cpu.registers.acc = 0;
        cpu.registers.e = 0;
        cpu.emulate();
        assert_eq!(cpu.flags.zero, true)
    }

    #[test]
    fn test_set_sign_flag() {
        let mut cpu = create_test_cpu(vec![0x83]);
        cpu.registers.acc = 0;
        cpu.registers.e = 10;
        cpu.flags.sign = true;
        cpu.emulate();
        assert_eq!(cpu.flags.sign, false)
    }

    #[test]
    fn test_subtraction() {
        let mut cpu = create_test_cpu(vec![0x97]);
        cpu.registers.acc = 0x3E;
        cpu.flags.carry = true;
        cpu.emulate();
        assert_eq!(cpu.registers.acc, 0);
        assert_eq!(cpu.flags.carry, false)
    }

    #[test]
    fn test_subtraction_with_carry() {
        let mut cpu = create_test_cpu(vec![0x98]);
        cpu.registers.acc = 10;
        cpu.registers.b = 3;
        cpu.flags.carry = true;
        cpu.emulate();
        assert_eq!(cpu.registers.acc, 6);
        assert_eq!(cpu.flags.carry, false)
    }

    #[test]
    fn test_sub_carry_flag() {
        let mut cpu = create_test_cpu(vec![0x90]);
        cpu.registers.acc = 2;
        cpu.registers.b = 3;
        cpu.flags.carry = false;
        cpu.emulate();
        assert_eq!(cpu.flags.carry, true);
        assert_eq!(cpu.registers.acc, u8::max_value())
    }

    #[test]
    fn test_logical_and() {
        let mut cpu = create_test_cpu(vec![0xa0]);
        cpu.registers.acc = 0b11111100;
        cpu.registers.b = 0b00001111;
        cpu.emulate();
        assert_eq!(cpu.registers.acc, 0b00001100)
    }

    #[test]
    fn test_logical_xor() {
        let mut cpu = create_test_cpu(vec![0xa9]);
        cpu.registers.acc = 0b01011100;
        cpu.registers.c = 0b01111000;
        cpu.emulate();
        assert_eq!(cpu.registers.acc, 0b00100100)
    }

    #[test]
    fn test_logical_or() {
        let mut cpu = create_test_cpu(vec![0xb2]);
        cpu.registers.acc = 0b11111100;
        cpu.registers.d = 0b00001111;
        cpu.emulate();
        assert_eq!(cpu.registers.acc, 0b11111111)
    }
}
