use super::constants::*;
use super::dram;
use super::errors;
use super::uart;

pub struct Bus {
    pub dram: dram::Dram,
    pub uart: uart::Uart,
}

impl Bus {
    pub fn new(dram: dram::Dram) -> Bus {
        return Self { 
            dram: dram, 
            uart: uart::Uart::new() 
        };
    }

    pub fn load(&self, addr: u64, size: u64) -> Result<u64, errors::Exception> {
        match (addr) {
            DRAM_BASE..=DRAM_END => self.dram.load(addr, size),
            UART_BASE..=UART_END => self.uart.load(addr, size),
            _ => Err(errors::Exception::LoadAccessFault(addr)),
        }
    }

    pub fn store(&mut self, addr: u64, size: u64, value: u64) -> Result<(), errors::Exception> {
        match(addr) {
            DRAM_BASE..=DRAM_END => self.dram.store(addr, size, value),
            UART_BASE..=UART_END => self.uart.store(addr, size, value),
            _ => Err(errors::Exception::StoreAMOAccessFault(addr)),
        }
    }
}

