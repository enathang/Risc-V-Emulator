use super::dram;
use super::errors;

pub struct Bus {
    pub dram: dram::Dram,
}

impl Bus {
    pub fn new(dram: dram::Dram) -> Bus {
        return Self { dram: dram };
    }

    pub fn load(&self, addr: u64, size: u64) -> Result<u64, errors::Exception> {
        return self.dram.load(addr, size);
    }

    pub fn store(&mut self, addr: u64, size: u64, value: u64) -> Result<(), errors::Exception> {
        return self.dram.store(addr, size, value);
    }
}

