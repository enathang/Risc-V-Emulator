use super::dram;

pub struct Bus {
    pub dram: dram::Dram,
}

impl Bus {
    pub fn new(dram: dram::Dram) -> Bus {
        return Self { dram: dram };
    }

    pub fn load(&self, addr: u64, size: u64) -> u64 {
        return self.dram.load(addr, size).unwrap();
    }

    pub fn store(&mut self, addr: u64, size: u64, value: u64) {
        return self.dram.store(addr, size, value).unwrap();
    }
}

