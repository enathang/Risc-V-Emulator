use std::{error::Error, fmt};

#[derive(Debug)]
pub struct Thing;

impl Error for Thing {}

impl fmt::Display for Thing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Oh no, something bad went down")
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Exception {
    InstructionAddrMisaligned(u64),
    InstructionAccessFault(u64),
    IllegalInstruction(u64),
    Breakpoint(u64),
    LoadAccessMisaligned(u64),
    LoadAccessFault(u64),
    StoreAMOAddrMisaligned(u64),
    StoreAMOAccessFault(u64),
    EnvironmentCallFromUMode(u64),
    EnvironmentCallFromSMode(u64),
    EnvironmentCallFromMMode(u64),
    InstructionPageFault(u64),
    LoadPageFault(u64),
    StoreAMOPageFault(u64),
}

impl Exception {
    pub fn value(self) -> u64 {
        match self {
            Exception::InstructionAddrMisaligned(addr) => addr,
            Exception::InstructionAccessFault(addr) => addr,
            Exception::IllegalInstruction(instr) => instr,
            Exception::Breakpoint(pc) => pc,
            Exception::LoadAccessMisaligned(addr) => addr,
            Exception::LoadAccessFault(addr) => addr,
            Exception::StoreAMOAddrMisaligned(addr) => addr,
            Exception::StoreAMOAccessFault(addr) => addr,
            Exception::EnvironmentCallFromUMode(pc) => pc,
            Exception::EnvironmentCallFromSMode(pc) => pc,
            Exception::EnvironmentCallFromMMode(pc) => pc,
            Exception::InstructionPageFault(addr) => addr,
            Exception::LoadPageFault(addr) => addr,
            Exception::StoreAMOPageFault(addr) => addr,
        }
    }

    pub fn code(self) -> u64 {
        match(self) {
            Exception::InstructionAddrMisaligned(_) => 0,
            Exception::InstructionAccessFault(_) => 1,
            Exception::IllegalInstruction(_) => 2,
            Exception::Breakpoint(_) => 3,
            Exception::LoadAccessMisaligned(_) => 4,
            Exception::LoadAccessFault(_) => 5,
            Exception::StoreAMOAddrMisaligned(_) => 6,
            Exception::StoreAMOAccessFault(_) => 7,
            Exception::EnvironmentCallFromUMode(_) => 8,
            Exception::EnvironmentCallFromSMode(_) => 9,
            Exception::EnvironmentCallFromMMode(_) => 11,
            Exception::InstructionPageFault(_) => 12,
            Exception::LoadPageFault(_) => 13,
            Exception::StoreAMOPageFault(_) => 15,
        }
    }

    pub fn is_fatal(self) -> bool {
        match(self) {
            Exception::InstructionAddrMisaligned(_)
            | Exception::InstructionAccessFault(_)
            | Exception::LoadAccessFault(_)
            | Exception::StoreAMOAddrMisaligned(_)
            | Exception::StoreAMOAccessFault(_) 
            | Exception::IllegalInstruction(_) => true,
            _else => false,
        }
    }
}
