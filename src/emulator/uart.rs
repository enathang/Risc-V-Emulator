use std::io;
use std::io::prelude::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Condvar, Mutex
};
use std::thread;
use super::errors;

pub const UART_SIZE: u64 = 128;

// RHR is the Receiver Holding Register (a register holding input bytes)
// THE UART_RHR_INDEX is the index of the RHR register in Uart
pub const UART_RHR_INDEX: u64 = 0;

// THR is the Transmissions Holding Register (same as RHR but for output)
// My best understanding is because UART has single signal, the same register
//    is used for both input and output, with mode being determined by LSR flags.
pub const UART_THR_INDEX: u64 = 0;

// LSR is the Line Status Register (a register storing the status of Uart)
// Bit 0 is 1: data is stored in RHR for processing (0 is empty)
// Bit 5 is 0: THR is full and waiting to be sent out (1 is empty)
pub const UART_LSR_INDEX: u64 = 5;
pub const UART_LSR_RHR_STATUS_FLAG: u8 = 1;
pub const UART_LSR_THR_STATUS_FLAG: u8 = 1 << 5;

pub const UART_BASE: u64 = 0x1000_0000;

pub struct Uart {
    uart: Arc<(Mutex<[u8; UART_SIZE as usize]>, Condvar)>,
    interrupt: Arc<AtomicBool>
}

impl Uart {
    pub fn new() -> Self {
        let mut array = [0; UART_SIZE as usize];
        let uart = Arc::new((Mutex::new(array), Condvar::new()));

        let interrupt = Arc::new(AtomicBool::new(false));

        Self::spawn_io_listener_thread(&uart, &interrupt);

        return Uart { uart: uart, interrupt: interrupt};
    }

    fn spawn_io_listener_thread(uart: &Arc<(Mutex<[u8; 128]>, Condvar)>, interrupt: &Arc<AtomicBool>) {
        let mut byte = [0];

        // Create reference to Uart for IO to load data into
        let read_uart = Arc::clone(&uart);
        let read_interrupt = Arc::clone(&interrupt);

        // Create a thread that continuously reads io
        thread::spawn(move || loop {
            let read_io = io::stdin().read(&mut byte);
            match(read_io) {
                Ok(_) => {
                    let (uart, cvar) = &*read_uart;
                    let mut array = uart.lock().unwrap();

                    // While the receiver flag (index 0) is set
                    while (array[UART_LSR_INDEX as usize] & UART_LSR_RHR_STATUS_FLAG == 1) {
                        // Wait and reload the status register
                        array = cvar.wait(array).unwrap();
                    }

                    // Receiver flag is 0, so load next one
                    array[UART_RHR_INDEX as usize] = byte[0];
                    read_interrupt.store(true, Ordering::Release);
                    array[UART_LSR_INDEX as usize] |= UART_LSR_RHR_STATUS_FLAG; // Maybe should be at beginning?
                },
                Err(e) => println!("Error: {}", e),
            }
        });
    }

    pub fn load(&self, addr: u64, size: u64) -> Result<u64, errors::Exception> {
        let (uart, cvar) = &*self.uart;
        let mut array = uart.lock().unwrap(); // Must be mut because we reset flag
        let index = addr - UART_BASE;

        match (index) {
            UART_RHR_INDEX => {
                cvar.notify_one();
                array[UART_LSR_INDEX as usize] &= !1; // Reset flag
                return Ok(array[index as usize] as u64);
            },
            _ => return Ok(array[index as usize] as u64),
        }
    }

    pub fn store(&mut self, addr: u64, size: u64, value: u64) -> Result<(), errors::Exception> {
        let (uart, cvar) = &*self.uart;
        let mut array = uart.lock().unwrap();

        let index = addr - UART_BASE;

        match (index) {
            UART_THR_INDEX => {
                print!("{}", value as u8 as char);
                io::stdout().flush().unwrap();
                return Ok(());
            }
            _ => {
                array[index as usize] = value as u8;
                return Ok(());
            }
        }
    }

    pub fn is_interrupting(&self) -> bool {
        return self.interrupt.swap(false, Ordering::Acquire);
    }
}
