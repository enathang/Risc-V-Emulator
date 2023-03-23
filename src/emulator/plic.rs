use super::errors;

pub const NUM_INPUTS: u64 = 64; // Defining our architecture to support 64 inputs max, because it
// can fit all flags on one u64

pub struct Plic {
    spriority: [u64; NUM_INPUTS as usize],
    pending: u64,
    claim: u64,
    completed: u64,
}

pub const PLIC_BASE: u64 = 0x1000_0000;
pub const PLIC_SPRIORITY_ADDR: u64 = PLIC_BASE+4;
pub const PLIC_SPRIORITY_END: u64 = PLIC_SPRIORITY_ADDR+(NUM_INPUTS*8*8)-1;
pub const PLIC_SPENDING_ADDR: u64 = PLIC_SPRIORITY_END + 4;
pub const PLIC_CLAIM_ADDR: u64 = PLIC_SPENDING_ADDR + 4;
pub const PLIC_COMPLETED_ADDR: u64 = PLIC_CLAIM_ADDR + 4;
pub const PLIC_END: u64 = PLIC_COMPLETED_ADDR;

impl Plic {
    pub fn new() -> Self {
        let spriority = [0; NUM_INPUTS as usize];
        return Plic {
            spriority : spriority,
            pending : 0,
            claim : 0,
            completed : 0,
        };
    }

    pub fn load(&self, addr: u64) -> Result<u64, errors::Exception> {
        match(addr) {
            PLIC_SPRIORITY_ADDR..=PLIC_SPRIORITY_END => {
                let index = (addr - PLIC_SPRIORITY_ADDR) / 64;
                return Ok(self.spriority[index as usize]);
            },
            PLIC_SPENDING_ADDR => Ok(self.pending),
            PLIC_CLAIM_ADDR => Ok(self.claim),
            PLIC_COMPLETED_ADDR => Ok(self.completed),
            _ => Err(errors::Exception::LoadAccessFault(addr))
        }
    }

    pub fn store(&mut self, addr: u64, value: u64) -> Result<(), errors::Exception> {
        match(addr) {
            PLIC_SPRIORITY_ADDR..=PLIC_SPRIORITY_END => {
                let index = (addr - PLIC_SPRIORITY_ADDR) / 64;
                return Ok(self.spriority[index as usize] = value);
            },
            PLIC_SPENDING_ADDR => Ok(self.pending = value),
            PLIC_CLAIM_ADDR => Ok(self.claim = value),
            PLIC_COMPLETED_ADDR => Ok(self.completed = value),
            _ => Err(errors::Exception::StoreAMOPageFault(addr)),
        }
    }

}
