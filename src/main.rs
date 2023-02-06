use std::io::prelude::*;
use std::env;
use std::fs::File;
use std::io;

fn main() -> io::Result<()> {
    println!("Hello, world!");

    // Load program from file into memory
    let args: Vec<String> = env::args().collect();

    if (args.len() != 2) {
        panic!("Usage: emulator <filename>");
    }

    let mut file = File::open(&args[1])?;
    let mut code = Vec::new();
    file.read_to_end(&mut code);
    
    // Create Cpu and load instructions into program memory
    let mut cpu = Cpu::new(code);
    
    // Start instruction fetch-decode-execute loop
    while (true) {
        let instr = cpu.fetch();
        let instr_decoded = cpu.decode(instr);
        cpu.execute(instr_decoded);
    // Break if end of memory is reached
    }

    Ok(())
}

struct R_Instr {
    opcode: u32,
    rd: usize,
    funct3: usize,
    rs1: usize,
    rs2: usize,
    funct7: usize,
}

struct Cpu {
    regs: [u64; 32],
    pc: u64,
    dram: Vec<u8>,
}

impl Cpu {

    fn new(code: Vec<u8>) -> Self {
        let mut cpu = Self { 
            regs: [0; 32], 
            pc: 0, 
            dram: code,
        };

        let MEMORY_SIZE = 1024*1024*128; // Define 10MiB memory
        cpu.regs[2] = MEMORY_SIZE; // Set stack pointer to end of memory (because it expands upwards)
        cpu.regs[0] = 0;  // Set zero register to 0s

        return cpu;
    }

    fn run(&mut self) {
        while (self.pc < self.dram.len() as u64) {
            let instr = self.fetch();
            let instr_decoded = self.decode(instr);
            self.pc = self.pc + 4; // Occurs before self.execute() in case op overwrites the pc
            self.execute(instr_decoded);
        }
    }

    fn fetch(&self) -> u32 {
        // Read 32 bit instruction from memory
        let index = self.pc as usize;
        // Use lower endianness
        let instr = (self.dram[index] as u32) | ((self.dram[index + 1] as u32) << 8) | ((self.dram[index + 2] as u32) << 16) | ((self.dram[index+3] as u32) << 24);
        
        return instr;
    }

    fn decode(&self, inst: u32) -> R_Instr {
        // Decode instruction
        let op: u32 = inst & 0x7f; // Note: 0x7f -> 0b0111_1111 , which acts as a mask
        let rd = ((inst >> 7) & 0x1f) as usize;
        let funct3 = ((inst >> 12) & 0x7) as usize;
        let rs1 = ((inst >> 15) & 0x1f) as usize;
        let rs2 = ((inst >> 20) & 0x1f) as usize;
        let funct7 = ((inst >> 25) & 0x7f) as usize;
        let instr_decoded = R_Instr { opcode: op, rd: rd, funct3: funct3, rs1: rs1, rs2: rs2, funct7: funct7 };

        return instr_decoded;
    }

    fn execute(&mut self, inst: R_Instr) {
        // Execute instruction 
        match(inst.opcode) {
            0x33 => { // ADD (Add rs1 and rs2)
                // TODO: Worry about overflow from addition
                self.regs[inst.rd] = self.regs[inst.rs1] + self.regs[inst.rs2];
            }
            0x13 => { // ADDI (Add rs1 and intermediate value from register)
                let imm: u64 = ((inst.funct7 as u64) << 4) | inst.rs2 as u64; // Imm value is stored as [funct7][rs2]
                // TODO: Worry about overflow from addition
                self.regs[inst.rd] = imm + self.regs[inst.rs1];
            }
            _ => {
                panic!("Op {0} not implemented yet!", inst.opcode);
            }
        }
    }
}

