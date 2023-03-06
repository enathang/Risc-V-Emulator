/// Machine status register.
pub const MSTATUS: usize = 0x300;
/// Machine interrupt delefation register.
pub const MIDELEG: usize = 0x303;
/// Machine interrupt-enable register.
pub const MIE: usize = 0x304;
/// Machine interrupt pending.
pub const MIP: usize = 0x344;

// Supervisor-level CSRs.
/// Supervisor status register.
pub const SSTATUS: usize = 0x100;
/// Supervisor interrupt-enable register.
pub const SIE: usize = 0x104;
/// Supervisor interrupt pending.
pub const SIP: usize = 0x144;

// mstatus and sstatus field mask
pub const MASK_SIE: u64 = 1 << 1; 
pub const MASK_SPIE: u64 = 1 << 5; 
pub const MASK_UBE: u64 = 1 << 6; 
pub const MASK_SPP: u64 = 1 << 8; 
pub const MASK_FS: u64 = 0b11 << 13; 
pub const MASK_XS: u64 = 0b11 << 15; 
pub const MASK_SUM: u64 = 1 << 18; 
pub const MASK_MXR: u64 = 1 << 19; 
pub const MASK_UXL: u64 = 0b11 << 32; 
pub const MASK_SD: u64 = 1 << 63; 
pub const MASK_SSTATUS: u64 = MASK_SIE | MASK_SPIE | MASK_UBE | MASK_SPP | MASK_FS 
                            | MASK_XS  | MASK_SUM  | MASK_MXR | MASK_UXL | MASK_SD;

pub struct Csr {
    csrs: [u64; 4096],
}

impl Csr {
    pub fn new() -> Csr {
        Self { csrs: [0; 4096] }
    }

    pub fn load(&self, addr: usize) -> u64 {
        match addr {
            SIE => self.csrs[MIE] & self.csrs[MIDELEG],
            SIP => self.csrs[MIP] & self.csrs[MIDELEG],
            SSTATUS => self.csrs[MSTATUS] & MASK_SSTATUS,
            _ => self.csrs[addr],
        }
    }

    pub fn store(&mut self, addr: usize, value: u64) {
        match addr {
            SIE => self.csrs[MIE] = (self.csrs[MIE] & !self.csrs[MIDELEG]) | (value & self.csrs[MIDELEG]),
            SIP => self.csrs[MIP] = (self.csrs[MIE] & !self.csrs[MIDELEG]) | (value & self.csrs[MIDELEG]),
            SSTATUS => self.csrs[MSTATUS] = (self.csrs[MSTATUS] & !MASK_SSTATUS)| (value & MASK_SSTATUS),
            _ => self.csrs[addr] = value,
        }
    }
}
