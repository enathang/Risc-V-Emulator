mod bus;
mod csr;
mod dram;
mod errors;
mod instructions;

type Mode = u64;
const User: Mode = 0; // 0b00
const Supervisor: Mode = 1; // 0b01
const Machine: Mode = 3; // 0b11

pub struct Cpu {
    pub regs: [u64; 32],
    pub pc: u64,
    pub bus: bus::Bus,
    pub csr: csr::Csr,
    pub mode: Mode,
}

impl Cpu {

    pub fn new(code: Vec<u8>) -> Self {
        let dram = dram::Dram::new(code);
        let bus = bus::Bus::new(dram);
        let csr = csr::Csr::new();
        let mode = Machine;
        let mut cpu = Self { 
            regs: [0; 32], 
            pc: 0, 
            bus: bus,
            csr: csr,
            mode: mode,
        };

        let MEMORY_SIZE = 1024*1024*128; // Define 10MiB memory
        cpu.regs[2] = MEMORY_SIZE; // Set stack pointer to end of memory (because it expands upwards)
        cpu.regs[0] = 0;  // Set zero register to 0s

        return cpu;
    }

    pub fn run(&mut self) {
        while (self.pc < self.bus.dram.dram.len() as u64) {
            let instr = self.fetch();
            let instr_decoded = self.decode(instr);
            let new_pc = self.execute(instr_decoded);
            self.pc = new_pc;
        }
    }

    fn fetch(&self) -> u32 {
        // Read 32 bit instruction from memory
        let index = self.pc as usize;
        let instr = self.bus.load((index as u64), 32);
        //let instr = (self.dram[index+3] as u32) | ((self.dram[index + 2] as u32) << 8) | ((self.dram[index + 1] as u32) << 16) | ((self.dram[index+0] as u32) << 24);
        
        return instr as u32;
    }

    fn decode(&self, inst: u32) -> instructions::R_Instr {
        println!("Decoding inst {:b}", inst);
        // Decode instruction
        let op: u32 = inst & 0x7f; // Note: 0x7f -> 0b0111_1111 , which acts as a mask
        let rd = ((inst >> 7) & 0x1f) as usize;
        let funct3 = ((inst >> 12) & 0x7) as usize;
        let rs1 = ((inst >> 15) & 0x1f) as usize;
        let rs2 = ((inst >> 20) & 0x1f) as usize;
        let funct7 = ((inst >> 25) & 0x7f) as usize;
        let instr_decoded = instructions::R_Instr { opcode: op, rd: rd, funct3: funct3, rs1: rs1, rs2: rs2, funct7: funct7 };

        println!("Decoded as {:?}", instr_decoded);
        return instr_decoded;
    }

    pub fn dump_registers(&self) {
        for i in (0..self.regs.len()) {
            println!("RegisterNum: {}, RegisterValue: {}", i, self.regs[i]);
        }
    }

    pub fn handle_error(&mut self, error: errors::Exception) {
        let mode = self.mode;
        let pc = self.pc;

        // Update privilege level
        // - Check level's medeleg to see if should be s or m
        let medeleg = self.csr.load(csr::MEDELEG);
        let exception_index = error.code();
        let should_deleg_to_supervisor = mode != Machine && (medeleg >> exception_index) & 1 == 1;

        if (should_deleg_to_supervisor) {
            self.mode = Supervisor;
            // Save PC
            self.csr.store(csr::SEPC, pc);
            // Update PC to trap handler
            self.pc = self.csr.load(csr::STVEC);

            self.csr.store(csr::SCAUSE, error.code());
            self.csr.store(csr::STVAL, error.value());
        
            let mut status = self.csr.load(csr::SSTATUS);
            let ie = status & csr::MASK_SIE >> 1;
            // First, we clear the flag bit, then we set it to new value
            status = status & !csr::MASK_SPIE | (ie << 1);
            let spp = mode;
            status = status & !csr::MASK_SPP | (spp << 8);
            self.csr.store(csr::SSTATUS, status);

        } else { 
            self.mode = Machine;
            // Save PC
            self.csr.store(csr::MEPC, pc);
            // Update PC to trap handler
            self.pc = self.csr.load(csr::MTVEC);
            
            self.csr.store(csr::MCAUSE, error.code());
            self.csr.store(csr::MTVAL, error.value());
            
            let mut status = self.csr.load(csr::MSTATUS);
            let ie = status & csr::MASK_MIE >> 3;
            status = status & !csr::MASK_MPIE | (ie << 3);
            let mpp = mode;
            status = status & !csr::MASK_MPP | (mpp << 11);
            self.csr.store(csr::MSTATUS, status);
        }
    }

    fn execute(&mut self, inst: instructions::R_Instr) -> u64 {
        // Execute instruction 
        match(inst.opcode) {
            0x33 => { // ADD (Add rs1 and rs2)
                // TODO: Worry about overflow from addition
                self.regs[inst.rd] = self.regs[inst.rs1] + self.regs[inst.rs2];
                return self.pc + 4;
            }
            0x13 => { // ADDI (Add rs1 and intermediate value from register)
                let imm: u64 = ((inst.funct7 as u64) << 5) | inst.rs2 as u64; // Imm value is stored as [funct7][rs2]
                // TODO: Worry about overflow from addition
                self.regs[inst.rd] = imm + self.regs[inst.rs1];
                return self.pc + 4;
            }
            0x37 => { // LUI (load 12-31 bits into register)
                let imm = ((inst.funct7 as u64) << 13) | ((inst.rs2 as u64) << 8) | ((inst.rs1 as u64) << 3) | inst.funct3 as u64;
                self.regs[inst.rd] = (imm << 12);
                return self.pc + 4;
            }
            0x73 => { // Diff CSR instructions have the same OP code, but differing rs2/funct7
                match(inst.rs2, inst.funct7) {
                    (0x3, _) => { // csrrc
                        let csr = (inst.funct7 << 5) | inst.rs2;
                        let temp = self.csr.load(csr);
                        let csr_value = temp & !self.regs[inst.rs1];
                        self.csr.store(csr, csr_value);
                        self.regs[inst.rd] = temp;
                        return self.pc + 4;
                    }
                    (0x2, 0x8) => { // sret
                        // Below is just fancy bit manipulation of sstatus to update certain flags
                        let mut updated_sstatus = self.csr.load(csr::SSTATUS);
                        
                        // Set the current mode to be the SPP (supervisor previous privilege) bit,
                        // which is either 0 for user or 1 for supervisor
                        let SPP_FLAG_POS = 8; 
                        self.mode = (updated_sstatus & (1 << SPP_FLAG_POS) >> SPP_FLAG_POS);
                        
                        // Set current IE (interrupt enabled) flag to be previous IE flag before
                        // interrupt 
                        let SPIE_FLAG_POS = 5;
                        let SIE_FLAG_POS = 1;
                        let spie = (updated_sstatus & (1 << SPIE_FLAG_POS) >> SPIE_FLAG_POS);
                        updated_sstatus = (updated_sstatus & !(1 << SIE_FLAG_POS)) | (spie << SIE_FLAG_POS);
                        
                        // Set Previous IE to be 1
                        updated_sstatus |= (1 << SPIE_FLAG_POS);
                    
                        // Set previous privilege mode to be user mode (which is lowest privilege)
                        updated_sstatus &= !(1 << SPP_FLAG_POS);
                        self.csr.store(csr::SSTATUS, updated_sstatus);

                        // Return the program counter position before interrupt, to restore program
                        let SEPC = 0x141; 
                        return self.csr.load(SEPC) & !0b11;
                    }
                    (_, _) => {
                        println!("CSR instruction not supported yet!");
                        return self.pc + 4;
                    }
                }
            }
            _ => {
                println!("Not implemented");
                println!("Op {} not implemented yet!", inst.opcode);
                return self.pc + 4;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_add_decode() {
        // Op = ADD, rd = 29, rs1 = 0, rs2 = 5
        let inst = format!("{}{}{}{}{}{}", "0000000", "00101", "00000", "000", "11101", "0010011");
        let inst_bin = u32::from_str_radix(&inst, 2).unwrap();
        let mut cpu = Cpu::new(Vec::new());
        let inst_obj = cpu.decode(inst_bin);

        let expected_instr = instructions::R_Instr{ opcode: 0x13, rd: 29, funct3: 0, rs1: 0, rs2: 5, funct7: 0 };
        assert_eq!(inst_obj, expected_instr);
    }

    #[test]
    fn test_execute_lui() {
        let mut cpu = Cpu::new(Vec::new());
        assert_eq!(0, cpu.regs[29]);

        let inst = format!("{}{}{}{}{}{}", "1111111", "11111", "11111", "111", "11101", "0110111");
        let inst_bin = u32::from_str_radix(&inst, 2).unwrap();
        let inst_obj = cpu.decode(inst_bin);
        cpu.execute(inst_obj);

        let expected_register_value = format!("{}{}{}{}{}{}", "1111111", "11111", "11111", "111", "00000", "0000000");
        let expected_register_value_bin = u32::from_str_radix(&expected_register_value, 2).unwrap();

        assert_eq!(expected_register_value_bin as u64, cpu.regs[29]);
    }

    #[test]
    fn test_execute_csrrc() {
        let mut cpu = Cpu::new(Vec::new());
        cpu.csr.store(3, 1);

        let inst = format!("{}{}{}{}{}", "000000000011", "00010", "011", "00001", "1110011");
        let inst_bin = u32::from_str_radix(&inst, 2).unwrap();
        let inst_obj = cpu.decode(inst_bin);
        cpu.execute(inst_obj);

        let expected_csr_value = 1;
        let expected_reg_value = 1;

        assert_eq!(expected_csr_value, cpu.csr.load(3));
        assert_eq!(expected_reg_value, cpu.regs[1]);
    }
}
