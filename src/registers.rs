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
}

impl Flags {
    pub fn new() -> Flags {
        Flags {
            zero: false,
            sign: false,
            parity: false,
            carry: false,
        }
    }

    pub fn set_all(&mut self, value: u16) {
        self.zero = value.count_ones() == 0;
        self.sign = (value & 0x80) != 0;
        self.parity = value.count_ones() % 2 == 0; // or 0
        self.carry = value > 0xFF;
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_get_hl() {
        let mut registers = Registers::new();
        registers.h = 0b10101010;
        registers.l = 0b01010101;
        assert_eq!(registers.get_hl(), 0b1010101001010101)
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