use std::error::Error;
use super::errors;

pub struct Dram {
    pub dram: Vec<u8>,
}

impl Dram {
    pub fn new(code: Vec<u8>) -> Dram {
        let mut dram = vec![0; 1024*1024*128 as usize];
        dram.splice(..code.len(), code.into_iter());
        Self { dram }
    }

    pub fn load(&self, addr: u64, size: u64) -> Result<u64, errors::Thing> {
        if (![8, 16, 32, 64].contains(&size)) {
            panic!("LoadAccessFault {}", addr);
        }

        let nbytes = size / 8;
        let index = addr as usize;
        let mut code = self.dram[index] as u64;
        for offset in 1..nbytes as usize {
            code = (code << 8) | (self.dram[index+offset] as u64); // TODO: Switch to small
            // endianness
        }

        return Ok(code);
    }

    pub fn store(&mut self, addr: u64, size: u64, data: u64) -> Result<(), errors::Thing> {
        if (![8, 16, 32, 64].contains(&size)) {
            panic!("LoadAccessFault {}", addr);
        }

        let nbytes = size / 8;
        let index = addr as usize - self.dram.len();
        for offset in 0..nbytes {
            self.dram[index+offset as usize] = ((data >> (8*offset)) & 0xff) as u8;
        }
        
        return Ok(());
    }
}


