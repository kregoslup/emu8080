use std::path::Path;
use std::fs::File;
use std::io::Read;
use dirs::home_dir;
use std::fmt::Debug;

pub const N_BYTES: usize = 65536;

#[derive(Debug)]
pub struct Memory {
    mapping: Vec<u8>,
}

impl Memory {
    pub fn load_rom(path: &Path) -> Memory {
        let loaded_rom: Vec<u8> = read_file(path);
        return Memory::new(loaded_rom)
    }

    pub fn instructions_len(&self) -> usize {
        self.mapping.len()
    }

    pub fn fetch_byte_at_offset(&self, pointer: u16) -> u8 {
        return self.mapping[pointer as usize]
    }

    pub fn fetch_bytes_at_offset(&self, pointer: u16, size: usize) -> &[u8] {
        return &self.mapping[(pointer as usize)..(pointer as usize) + size]
    }

    pub fn set_byte_at_offset(&mut self, pointer: u16, value: u8) {
        self.mapping[(pointer as usize)] = value;
    }

    pub fn new(memory: Vec<u8>) -> Memory {
        Memory {
            mapping: memory,
        }
    }
}

fn read_file(path: &Path) -> Vec<u8> {
    let mut file = File::open(path).unwrap();
    let mut data = Vec::new();
    file.read_to_end(&mut data);
    return data;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_file() {
        let mut tmp_dir = home_dir().unwrap();
        tmp_dir.push(".bash_history");
        assert_ne!(read_file(tmp_dir.as_path()).len(), 0)
    }

    #[test]
    fn test_load_rom_not_empty() {
        let mut tmp_dir = home_dir().unwrap();
        tmp_dir.push(".bash_history");
        let memory = Memory::load_rom(tmp_dir.as_path());
        assert_ne!(memory.fetch_byte_at_offset(0), 0)
    }

    #[test]
    fn test_set_byte() {
        let mut tmp_dir = home_dir().unwrap();
        tmp_dir.push(".bash_history");
        let mut memory = Memory::load_rom(tmp_dir.as_path());
        let offset = 0;
        let val = 20;
        memory.set_byte_at_offset(offset, val);
        assert_eq!(memory.fetch_byte_at_offset(offset), val)
    }
}