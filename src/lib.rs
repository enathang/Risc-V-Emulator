use std::io::prelude::*;
use std::env;
use std::fs::File;
use std::io;

pub mod emulator;

fn main() -> io::Result<()> {
    println!("Running Risc-V emulator!");

    // Load program from file into memory
    let args: Vec<String> = env::args().collect();

    if (args.len() != 2) {
        panic!("Usage: emulator <filename>");
    }

    let mut file = File::open(&args[1])?;
    let mut code = Vec::new();
    file.read_to_end(&mut code);

    println!("Contents! {:?}", code);

    // Create Cpu and load instructions into program memory
    let mut cpu = emulator::Cpu::new(code);
    
    // Start instruction fetch-decode-execute loop
    cpu.run();

    cpu.dump_registers();

    Ok(())
}

