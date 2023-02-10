extern crate Risc_V_Emulator;

use std::io::prelude::*;
use std::fs::File;
use std::env;
use std::io;

use Risc_V_Emulator::emulator;

fn read_binary(program_memory: &mut Vec<u8>, file_name: &str) -> io::Result<()> {
    let mut file = File::open(&file_name)?;
    file.read_to_end(program_memory)?;
    
    Ok(())
}

#[test]
fn test_add() {
    let mut code = Vec::new();
    let prog_file = "tests/binaries-for-testing/add-addi.bin";

    let result = read_binary(&mut code, &prog_file);
    println!("Result: {:?}", code);

    // assert_eq!(10, code.len()); // Sanity check that instructions were loaded correctly

    let mut cpu = emulator::Cpu::new(code);
    cpu.run();

    let mut expected_regs = [0; 32];
    expected_regs[2] = 1024*1024*128;
    expected_regs[29] = 5;
    expected_regs[30] = 37;
    expected_regs[31] = 42;

    assert_eq!(expected_regs, cpu.regs)
}

