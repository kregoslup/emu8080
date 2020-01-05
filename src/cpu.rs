use crate::registers::Registers;
use crate::registers::Flags;
use crate::memory::{Memory, N_BYTES};
use crate::op_code::OpCode;
use std::fmt::Debug;
use std::num::Wrapping;
use std::u8;
use std::process::exit;

#[derive(Debug)]
struct Cpu {
    stack_pointer: u16,
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
            0x00 => {
                println!("NOP code");
                self.program_counter += 1;
            },
            0x37 => {
                self.flags.carry = true;
                self.program_counter += 1;
            },
            0x3f => {
                self.flags.carry = !self.flags.carry;
                self.program_counter += 1;
            },
            0x76 => self.halt(),
            0x0a | 0x1a => self.load_accumulator(op_code),
            // STA
            0x32 => self.store_acc_direct(op_code),
            // LDA
            0x3a => self.load_acc_direct(op_code),
            0x01..=0x3e => self.single_operand_operation(op_code),
            0x40..=0x7f => self.transfer(op_code),
            0x80..=0xbf => self.arithmetic_operation(op_code),
            0xc5 | 0xd5 | 0xe5 | 0xf5 | 0xc6 | 0xe6 | 0xfe => self.single_operand_operation(op_code),
            0xc2 | 0xc3 | 0xca | 0xd2 | 0xda | 0xe2 | 0xea | 0xf2 | 0xfa => self.jump_to_address(op_code),
            0xeb => self.exchange_registers(op_code),
            _ => panic!("Unknown op code")
        }
    }

    fn jump_to_address(&mut self, op_code: &OpCode) {
        let instruction = op_code.extract_first_operand();
        let address = self.fetch_operand_addressed_memory();
        let result: bool = match instruction {
            0b0 => {
                let mut jmp = false;
                if op_code.extract_jmp_description() > 0 {
                    jmp = true;
                } else if !self.flags.zero {
                    jmp = true;
                }
                jmp
            },
            // TODO: Missing tests
            0b001 => self.flags.zero,
            0b010 => !self.flags.carry,
            0b011 => self.flags.carry,
            0b100 => !self.flags.parity,
            0b101 => self.flags.parity,
            0b110 => !self.flags.sign,
            0b111 => self.flags.sign,
            _ => panic!("Unknown jump description")
        };
        if result {
            self.program_counter = address;
        }
        self.program_counter += 1;
    }

    fn load_acc_direct(&mut self, op_code: &OpCode) {
        let address = self.fetch_operand_addressed_memory();
        let value = self.memory.fetch_byte_at_offset(address);
        self.registers.acc = value;
    }

    fn store_acc_direct(&mut self, op_code: &OpCode) {
        let address = self.fetch_operand_addressed_memory();
        self.memory.set_byte_at_offset(address, self.registers.acc);
    }

    fn fetch_operand_addressed_memory(&mut self) -> u16 {
        let msb = self.memory.fetch_byte_at_offset(self.program_counter + 2);
        let lsb = self.memory.fetch_byte_at_offset(self.program_counter + 1);
        self.program_counter += 2;
        (((msb as u16) << 8) | (lsb as u16))
    }

    fn exchange_registers(&mut self, op_code: &OpCode) {
        let mut tmp = self.registers.h;
        self.registers.h = self.registers.d;
        self.registers.d = tmp;
        tmp = self.registers.l;
        self.registers.l = self.registers.e;
        self.registers.e = tmp;
        self.program_counter += 1;
    }

    fn load_accumulator(&mut self, op_code: &OpCode) {
        let registry_pair = op_code.extract_registry_pair_description();
        let address;
        if registry_pair == 0 {
            address = self.registers.get_de()
        } else {
            address = self.registers.get_bc()
        }
        self.registers.acc = self.memory.fetch_byte_at_offset(address);
        self.program_counter += 1;
    }

    fn single_operand_operation(&mut self, op_code: &OpCode) {
        let operation = (op_code.extract_single_registry_operation(), op_code.extract_second_operand());
        let encoded_address = op_code.extract_first_operand();
        match operation {
            // INR
            (0b0, 0b100) => {
                let mut value = Wrapping(self.extract_memory_or_register(encoded_address));
                value += Wrapping(1);
                self.change_single_registry_value(encoded_address, value.0);
            },
            // DCR
            (0b0, 0b101) => {
                let mut value = Wrapping(self.extract_memory_or_register(encoded_address));
                value -= Wrapping(1);
                self.change_single_registry_value(encoded_address, value.0);
            },
            // ROTATE
            (0b0, 0b111) => {
                self.rotate_acc(encoded_address);
            },
            //DAD
            (0b0, 0b001) => {
                self.double_add(encoded_address);
            },
            // INX
            (0b0, 0b011) => {
                self.increment_double(encoded_address);
            },
            // PUSH
            (0b11, 0b101) => {
                self.push_on_stack(encoded_address);
            },
            // POP
            (0b11, 0b1) => {
                self.pop_off_stack(encoded_address);
            },
            // MVI
            (0b0, 0b110) => {
                self.move_immediate(encoded_address);
            },
            // ADI
            (0b11, 0b110) => {
                self.immediate_arithmetic(encoded_address);
            },
            _ => panic!("Unknown single registry operation")
        }

        self.program_counter += 1;
    }

    fn immediate_arithmetic(&mut self, operation: u8) {
        self.program_counter += 1;
        let data = self.memory.fetch_byte_at_offset(self.program_counter);
        match operation {
            0b0 => self.add(data, false),
            0b100 => self.and(data),
            0b111 => self.comparison(data),
            // TODO: Needs test
            0b001 => self.add(data, true),
            0b010 => self.subtract(data, false),
            0b011 => self.subtract(data, true),
            0b101 => self.xor(data),
            0b110 => self.or(data),
            _ => panic!("Unknown immediate operation")
        }
    }

    fn move_immediate(&mut self, address: u8) {
        self.program_counter += 1;
        let data = self.memory.fetch_byte_at_offset(self.program_counter);
        self.set_memory_or_register(address, data);
    }

    fn increment_double(&mut self, address: u8) {
        match address {
            0b0 => {
                let mut value = Wrapping(self.registers.get_bc());
                value += Wrapping(1);
                self.registers.set_bc(value.0);
            },
            0b010 => {
                let mut value = Wrapping(self.registers.get_de());
                value += Wrapping(1);
                self.registers.set_de(value.0);
            },
            0b100 => {
                let mut value = Wrapping(self.registers.get_hl());
                value += Wrapping(1);
                self.registers.set_hl(value.0);
            },
            0b110 => {
                let mut value = Wrapping(self.stack_pointer);
                value += Wrapping(1);
                self.stack_pointer = value.0;
            },
            _ => panic!("Unknown register pair")
        }
    }

    fn double_add(&mut self, address: u8) {
        let value;
        match address {
            0b001 => {
                value = self.registers.get_bc();
            },
            0b011 => {
                value = self.registers.get_de();
            },
            0b101 => {
                value = self.registers.get_hl();
            },
            0b111 => {
                value = self.stack_pointer;
            },
            _ => panic!("Unknown register pair")
        }
        let result: u32 = self.registers.get_hl() as u32 + value as u32;
        self.flags.set_carry_on_double(result);
        self.registers.set_hl(result as u16);
    }

    fn push_on_stack(&mut self, address: u8) {
        let upper;
        let lower;
        match address {
            0b0 => {
                upper = self.registers.b;
                lower = self.registers.c;
            },
            0b010 => {
                upper = self.registers.d;
                lower = self.registers.e;
            },
            0b100 => {
                upper = self.registers.h;
                lower = self.registers.l;
            },
            0b110 => {
                upper = self.registers.acc;
                lower = ((self.flags.sign as u8) << 7) |
                         ((self.flags.zero as u8) << 6) |
                         ((self.flags.aux_carry as u8) << 4) |
                         ((self.flags.parity as u8) << 2) |
                         (0b00000010) |
                        (self.flags.carry as u8)
            },
            _ => panic!("Unknown register pair")
        }
        self.memory.set_byte_at_offset(self.stack_pointer - 1, upper);
        self.memory.set_byte_at_offset(self.stack_pointer - 2, lower);
        self.stack_pointer -= 2;
    }

    fn pop_off_stack(&mut self, destination: u8) {
        let upper = self.memory.fetch_byte_at_offset(self.stack_pointer);
        let lower = self.memory.fetch_byte_at_offset(self.stack_pointer + 1);
        match destination {
            0b0 => {
                self.registers.b = lower;
                self.registers.c = upper;
            },
            0b010 => {
                self.registers.d = lower;
                self.registers.e = upper;
            },
            0b100 => {
                self.registers.h = lower;
                self.registers.l = upper;
            },
            0b110 => {
                self.registers.acc = lower;
                self.flags.sign = upper << 7 != 0;
                self.flags.zero = upper << 6 != 0;
                self.flags.aux_carry = upper << 4 != 0;
                self.flags.parity = upper << 2 != 0;
                self.flags.carry = upper != 0;
            },
            _ => panic!("Unknown register pair")
        }
        self.stack_pointer += 2;
    }

    fn rotate_acc(&mut self, direction: u8) {
        match direction {
            // RLC
            0b0 => {
                self.flags.carry = (self.registers.acc & 0b10000000) != 0;
                self.registers.acc = self.registers.acc.rotate_left(1)
            },
            // RRC
            0b01 => {
                self.flags.carry = (self.registers.acc & 0b00000001) != 0;
                self.registers.acc = self.registers.acc.rotate_right(1)
            }
            // missing RAL, RAR
            _ => panic!("Unknown rotation direction")
        }
    }

    fn change_single_registry_value(&mut self, encoded_address: u8, value: u8) {
        self.flags.set_single_registry_operation_flags(value as u16);
        self.set_memory_or_register(encoded_address, value)
    }

    fn transfer(&mut self, op_code: &OpCode) {
        let source = self.extract_source_value(op_code);
        let encoded_dest = op_code.extract_first_operand();
        self.set_memory_or_register(encoded_dest, source as u8);
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

    fn set_memory_or_register(&mut self, encoded_address: u8, value: u8) {
        if encoded_address == 0b110 {
            self.memory.set_byte_at_offset(self.registers.get_hl(), value);
        } else {
            *self.extract_register_address(encoded_address) = value;
        }
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
        unimplemented!();
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
            0b111 => self.comparison(value),
            _ => println!("Could not decode arithmetic operation")
        }
        self.program_counter += 1;
    }

    // TODO: Refactor - repetition
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

    fn comparison(&mut self, value: u8) {
        let result = Wrapping(self.registers.acc) - Wrapping(value);
        self.flags.set_all(result.0 as u16)
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
    // TODO: Wrap
    fn test_transfer_memory() {
        let result = 15;
        let mut cpu = create_test_cpu(vec![0x66, result]);
        cpu.registers.h = 0;
        cpu.registers.b = 2;
        cpu.emulate();
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

    #[test]
    fn test_increment() {
        let mut cpu = create_test_cpu(vec![0x04]);
        cpu.registers.b = 1;
        cpu.emulate();
        assert_eq!(cpu.registers.b, 2)
    }

    #[test]
    fn test_decrement() {
        let mut cpu = create_test_cpu(vec![0x0d]);
        cpu.registers.c = 0;
        cpu.emulate();
        assert_eq!(cpu.registers.c, 255);
        assert_eq!(cpu.flags.carry, false);
    }

    #[test]
    fn test_load_acc() {
        let mut cpu = create_test_cpu(vec![0xb2]);
        cpu.registers.acc = 0b11111100;
        cpu.registers.d = 0b00001111;
        cpu.emulate();
        assert_eq!(cpu.registers.acc, 0b11111111)
    }

    #[test]
    fn test_rotate_left() {
        let mut cpu = create_test_cpu(vec![0x07]);
        cpu.registers.acc = 0b11110010;
        cpu.flags.carry = false;
        cpu.emulate();
        assert_eq!(cpu.registers.acc, 0b11100101);
        assert_eq!(cpu.flags.carry, true);
    }

    #[test]
    fn test_rotate_right() {
        let mut cpu = create_test_cpu(vec![0x0F]);
        cpu.registers.acc = 0b11110010;
        cpu.flags.carry = true;
        cpu.emulate();
        assert_eq!(cpu.registers.acc, 0b01111001);
        assert_eq!(cpu.flags.carry, false);
    }

    #[test]
    fn test_push_on_stack() {
        let mut cpu = create_test_cpu(vec![0x00, 0xc5]);
        cpu.stack_pointer = 2;
        cpu.registers.b = 12;
        cpu.registers.c = 18;
        cpu.emulate();
        assert_eq!(cpu.memory.fetch_byte_at_offset(1), 12);
        assert_eq!(cpu.memory.fetch_byte_at_offset(0), 18);
        assert_eq!(cpu.stack_pointer, 0);
    }

    #[test]
    fn test_push_flags_on_stack() {
        let mut cpu = create_test_cpu(vec![0x00, 0xf5]);
        cpu.stack_pointer = 2;
        cpu.registers.acc = 12;
        cpu.flags.sign = true;
        cpu.flags.carry = true;
        cpu.flags.zero = false;
        cpu.flags.parity = true;
        cpu.emulate();
        assert_eq!(cpu.memory.fetch_byte_at_offset(1), 12);
        assert_eq!(cpu.memory.fetch_byte_at_offset(0), 0b10000111);
        assert_eq!(cpu.stack_pointer, 0);
    }

    #[test]
    fn test_double_add() {
        let mut cpu = create_test_cpu(vec![0x09]);
        cpu.registers.b = 0x33;
        cpu.registers.c = 0x9f;
        cpu.registers.h = 0xa1;
        cpu.registers.l = 0x7b;
        cpu.flags.carry = true;
        cpu.emulate();
        assert_eq!(cpu.registers.get_hl(), 0xd51a);
        assert_eq!(cpu.registers.h, 0xd5);
        assert_eq!(cpu.registers.l, 0x1a);
        assert_eq!(cpu.flags.carry, false);
    }

    #[test]
    fn test_double_increment() {
        let mut cpu = create_test_cpu(vec![0x13]);
        cpu.registers.d = 0x38;
        cpu.registers.e = 0xff;
        cpu.flags.carry = false;
        cpu.emulate();
        assert_eq!(cpu.registers.get_de(), 0x3900);
        assert_eq!(cpu.registers.d, 0x39);
        assert_eq!(cpu.registers.e, 0x00);
        assert_eq!(cpu.flags.carry, false);
    }

    #[test]
    fn test_double_increment_wraps() {
        let mut cpu = create_test_cpu(vec![0x33]);
        cpu.stack_pointer = 0xFFFF;
        cpu.emulate();
        assert_eq!(cpu.stack_pointer, 0x0);
    }

    #[test]
    fn test_exchange_registers() {
        let mut cpu = create_test_cpu(vec![0xeb]);
        cpu.registers.d = 1;
        cpu.registers.e = 2;
        cpu.registers.h = 3;
        cpu.registers.l = 4;
        cpu.emulate();
        assert_eq!(cpu.registers.d, 3);
        assert_eq!(cpu.registers.e, 4);
        assert_eq!(cpu.registers.h, 1);
        assert_eq!(cpu.registers.l, 2);
    }

    #[test]
    fn test_move_immediate_to_register() {
        let mut cpu = create_test_cpu(vec![0x3e, 15]);
        cpu.registers.acc = 12;
        cpu.emulate();
        assert_eq!(cpu.registers.acc, 15);
    }

    #[test]
    fn test_add_immediate() {
        let mut cpu = create_test_cpu(vec![0xc6, 12]);
        cpu.registers.acc = 12;
        cpu.emulate();
        assert_eq!(cpu.registers.acc, 24);
    }

    #[test]
    fn test_and_immediate() {
        let mut cpu = create_test_cpu(vec![0xe6, 0]);
        cpu.registers.acc = 0xFF;
        cpu.emulate();
        assert_eq!(cpu.registers.acc, 0);
    }

    #[test]
    fn test_immediate_comparison() {
        let mut cpu = create_test_cpu(vec![0x3e, 15]);
        cpu.registers.acc = 12;
        cpu.emulate();
        assert_eq!(cpu.registers.acc, 15);
    }

    #[test]
    fn test_store_direct() {
        let mut cpu = create_test_cpu(vec![0x32, 3, 0b0, 15]);
        cpu.registers.acc = 0;
        assert_eq!(cpu.memory.fetch_byte_at_offset(3), 15);
        cpu.emulate();
        assert_eq!(cpu.memory.fetch_byte_at_offset(3), 0);
    }

    #[test]
    fn test_load_direct() {
        let mut cpu = create_test_cpu(vec![0x3a, 3, 0b0, 0]);
        cpu.registers.acc = 15;
        cpu.emulate();
        assert_eq!(cpu.registers.acc, 0);
    }

    #[test]
    fn test_jmp() {
        let mut cpu = create_test_cpu(vec![0xc3, 3, 0b0, 0x04, 0]);
        cpu.emulate();
        assert_eq!(cpu.registers.b, 0);
    }

    #[test]
    fn test_jnz() {
        let mut cpu = create_test_cpu(vec![0xc2, 3, 0b0, 0x04, 0]);
        cpu.flags.zero = false;
        cpu.emulate();
        assert_eq!(cpu.registers.b, 0);
    }

    #[test]
    fn test_jnz_not_set() {
        let mut cpu = create_test_cpu(vec![0xc2, 3, 0b0, 0x04, 0]);
        cpu.flags.zero = true;
        cpu.emulate();
        assert_eq!(cpu.registers.b, 1);
    }
}
