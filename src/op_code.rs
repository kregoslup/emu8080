use std::convert::From;
use std::ops::BitAnd;

pub struct OpCode {
    pub value: u8,
}

impl OpCode {
    pub fn extract_registry_pair_description(&self) -> u8 {
        println!("Extracting single registry operation from {:#b}", self.value);
        (self.value.bitand(0b00010000)) >> 4
    }

    pub fn extract_single_registry_operation(&self) -> u8 {
        println!("Extracting single registry operation from {:#b}", self.value);
        (self.value.bitand(0b11000000)) >> 6
    }

    pub fn extract_first_operand(&self) -> u8 {
        println!("Extracting first operand from {:#b}", self.value);
        (self.value.bitand(0b00111000)) >> 3
    }

    pub fn extract_second_operand(&self) -> u8 {
        println!("Extracting second operand from {:#b}", self.value);
        self.value & 0b00000111
    }
}

impl From<u8> for OpCode {
    fn from(item: u8) -> Self {
        OpCode { value: item }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into_op_code() {
        let x: OpCode = 12_u8.into();
        assert_eq!(x.value, 12)
    }

    #[test]
    fn test_extract_source() {
        let x: OpCode = 0b00111000_u8.into();
        assert_eq!(x.extract_first_operand(), 0b111)
    }

    #[test]
    fn test_extract_dest() {
        let x: OpCode = 0b00111111_u8.into();
        assert_eq!(x.extract_second_operand(), 0b111)
    }
}