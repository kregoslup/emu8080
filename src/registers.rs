use std::u8;
use std::fmt::Debug;

#[derive(Debug)]
pub struct Registers {
    pub acc: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
}

#[derive(Debug)]
pub struct Flags {
    pub zero: bool,
    pub sign: bool,
    pub parity: bool,
    pub carry: bool,
    pub aux_carry: bool,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            acc: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
        }
    }

    pub fn get_hl(&self) -> u16 {
        (((self.h as u16) << 8) | (self.l as u16))
    }

    pub fn get_de(&self) -> u16 {
        (((self.d as u16) << 8) | (self.e as u16))
    }

    pub fn get_bc(&self) -> u16 {
        (((self.b as u16) << 8) | (self.c as u16))
    }
}

// TODO: AUX CARRY
impl Flags {
    pub fn new() -> Flags {
        Flags {
            zero: false,
            sign: false,
            parity: false,
            carry: false,
            aux_carry: false
        }
    }

    pub fn set_zero(&mut self, value: u16) {
        self.zero = value.count_ones() == 0;
    }

    pub fn set_parity(&mut self, value: u16) {
        self.parity = value.count_ones() % 2 == 0;
    }

    pub fn set_sign(&mut self, value: u16) {
        self.sign = (value & 0x80) != 0;
    }

    pub fn set_carry(&mut self, value: u16) {
        self.carry = value > 0xFF;
    }

    pub fn set_all(&mut self, value: u16) {
        self.set_zero(value);
        self.set_sign(value);
        self.set_parity(value);
        self.set_carry(value);
    }

    pub fn set_single_registry_operation_flags(&mut self, value: u16) {
        self.set_zero(value);
        self.set_sign(value);
        self.set_parity(value);
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_get_registry_pair() {
        let mut registers = Registers::new();
        let left = 0b10101010;
        let right = 0b01010101;
        let result = 0b1010101001010101;
        registers.h = left;
        registers.l = right;
        assert_eq!(registers.get_hl(), result);

        registers.d = left;
        registers.e = right;
        assert_eq!(registers.get_de(), result);

        registers.b = left;
        registers.c = right;
        assert_eq!(registers.get_bc(), result)
    }

    #[test]
    fn test_set_zero_flag() {
        let mut flags = Flags::new();
        flags.set_all(123);
        assert!(!flags.zero);

        flags.set_all(0);
        assert!(flags.zero);
    }

    #[test]
    fn test_set_sign_flag() {
        let mut flags = Flags::new();
        flags.set_all(0b10011001);
        assert!(flags.sign);

        flags.set_all(0b01011001);
        assert!(flags.sign)
    }

    #[test]
    fn test_set_parity_flag() {
        let mut flags = Flags::new();
        flags.set_all(0b10011001);
        assert!(flags.parity);

        flags.set_all(0b10011101);
        assert!(!flags.parity);
    }

    #[test]
    fn test_set_carry_flag() {
        let mut flags = Flags::new();
        flags.set_all(0b110011001);
        assert!(flags.carry);
    }
}