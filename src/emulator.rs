use self::csr::*;

mod bus;
mod csr;
mod constants;
mod dram;
mod errors;
mod instructions;
mod interrupt;
mod plic;
mod uart;

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
            let instr = match self.fetch() {
                Ok(instr) => instr as u32,
                Err(e) => {
                    self.handle_error(e);
                    continue;
                }
            };
            let instr_decoded = self.decode(instr);
            let new_pc = match self.execute(instr_decoded) {
                Ok(pc) => pc,
                Err(e) => {
                    self.handle_error(e);
                    continue;
                }
            };
            self.pc = new_pc;

            match self.check_pending_interrupt() {
                Some(interrupt) => self.handle_interrupt(interrupt),
                None => ()
            }
        }
    }

    fn fetch(&self) -> Result<u64, errors::Exception> {
        // Read 32 bit instruction from memory
        let index = self.pc as usize;
        let instr = self.bus.load((index as u64), 32);
        //let instr = (self.dram[index+3] as u32) | ((self.dram[index + 2] as u32) << 8) | ((self.dram[index + 1] as u32) << 16) | ((self.dram[index+0] as u32) << 24);
        
        return instr;
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

    pub fn handle_interrupt(&mut self, interrupt: interrupt::Interrupt) {
        let pc = self.pc;
        let mode = self.mode;
        let cause = interrupt.code();

        let delegate_to_s_mode = mode != Machine && (self.csr.load(csr::MIDELEG) & (1 << cause) > 0);
        if (delegate_to_s_mode) {
            // Determine whether CPU is setup to use vectorized or bare interrupt handling
            let tvec = self.csr.load(csr::STVEC);
            if (tvec & 0b11 == 0) {
                self.pc = tvec & !0b11;
            } else {
                self.pc = tvec & !0b11 + cause << 2;
            }

            // Store state before interrupt to restore later
            self.csr.store(csr::SEPC, pc);
            self.csr.store(csr::SCAUSE, cause);
            self.csr.store(csr::STVAL, 0);

            // Update state to handle interrupt
            let mut status = self.csr.load(csr::SSTATUS);
            status |= csr::MASK_SPIE; // If we're handing an interrupt, we can assume SIE=1
            status &= !csr::MASK_SIE;
            status = (status & !csr::MASK_SPP) | mode << 8;
            self.csr.store(csr::SSTATUS, status);
        } else {
           let tvec = self.csr.load(csr::MTVEC);
            if (tvec & 0b11 == 0) {
                self.pc = tvec & !0b11;
            } else {
                self.pc = tvec & !0b11 + cause << 2;
            }

            // Store state before interrupt to restore later
            self.csr.store(csr::MEPC, pc);
            self.csr.store(csr::MCAUSE, cause);
            self.csr.store(csr::MTVAL, 0);

            // Update state to handle interrupt
            let mut status = self.csr.load(csr::MSTATUS);
            status |= csr::MASK_MPIE; // If we're handing an interrupt, we can assume MIE=1
            status &= !csr::MASK_MIE;
            status = (status & !csr::MASK_MPP) | mode << 11;
            self.csr.store(csr::MSTATUS, status);
        }

    }

    pub fn handle_error(&mut self, error: errors::Exception) {
        if (error.is_fatal()) { 
            panic!("Fatal exception!");
        }

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

    pub fn check_pending_interrupt(&mut self) -> Option<interrupt::Interrupt> {
        // If MachineMode and machine interrupts disabled, ignore pending interrupts
        if (self.mode == Machine && (self.csr.load(MSTATUS) & MASK_MIE) == 0) {
            return None;
        }

        // If Supervisor mode and supervisor interrupts disabled, ignore pending interrupts
        if ((self.mode == Supervisor) &&  (self.csr.load(MSTATUS) & MASK_SIE) == 0) {
            return None;
        }

        if (self.bus.uart.is_interrupting()) {
            self.bus.store(plic::PLIC_CLAIM_ADDR, 32, uart::UART_IRQ).unwrap();
            self.csr.store(csr::MIP, self.csr.load(csr::MIP) | !csr::MASK_SEIP);
        }

        // Load a list of interrupts that are both enabled and pending
        let pending = self.csr.load(MIE) & self.csr.load(MIP);

        if (pending & MASK_MEIP) != 0 {
            self.csr.store(csr::MIP, self.csr.load(csr::MIP) & !MASK_MEIP);
            return Some(interrupt::Interrupt::MachineExternalInterrupt);
        }

        if (pending & MASK_MSIP) != 0 {
            self.csr.store(MIP, self.csr.load(MIP) & !MASK_MSIP);
            return Some(interrupt::Interrupt::MachineSoftwareInterrupt);
        }

        if (pending & MASK_MTIP) != 0 {
            self.csr.store(MIP, self.csr.load(MIP) & !MASK_MTIP);
            return Some(interrupt::Interrupt::MachineTimerInterrupt);
        }

        if (pending & MASK_SEIP) != 0 {
            self.csr.store(MIP, self.csr.load(MIP) & !MASK_SEIP);
            return Some(interrupt::Interrupt::SupervisorExternalInterrupt);
        }

        if (pending & MASK_SSIP) != 0 {
            self.csr.store(MIP, self.csr.load(MIP) & !MASK_SSIP);
            return Some(interrupt::Interrupt::SupervisorSoftwareInterrupt);
        }

        if (pending & MASK_STIP) != 0 {
            self.csr.store(MIP, self.csr.load(MIP) & !MASK_STIP);
            return Some(interrupt::Interrupt::SupervisorTimerInterrupt);
        }

        return None;
    }

    fn get_status_flag(status: &u64, flag_index: u64) -> bool {
        let flag_mask = (1 << flag_index);
        return ((status & flag_mask) > 0);
    }

    // TODO: Update this function
    /*fn set_status_flag(status: &mut u64, flag_index: u64, val: bool) {
        // First, reset the flag to 0
        status = status & !(1 << flag_index);

        // Then, set the flag to val
        status |= (val << flag_index);
    }*/

    fn execute(&mut self, inst: instructions::R_Instr) -> Result<u64, errors::Exception> {
        // Execute instruction 
        match(inst.opcode) {
            0x33 => {
                match (inst.funct3) {
                    0x0 => {
                        match (inst.funct7) {
                            0x0 => { // ADD
                                self.regs[inst.rd] = self.regs[inst.rs1] + self.regs[inst.rs2];
                            },
                            _ => { // SUB
                                self.regs[inst.rd] = self.regs[inst.rs2] - self.regs[inst.rs1];
                            },
                        }
                    }
                    _ => {}
                }
            }
            0x13 => {
                let imm = ((inst.funct7 as u64) << 5) | inst.rs2 as u64;
                match(inst.funct3) {
                    0x0 => { // ADDI
                        // TODO: Worry about overflow from addition
                        self.regs[inst.rd] = imm + self.regs[inst.rs1];
                        return Ok(self.pc + 4);
                    },
                    0x1 => { // SLLI
                        let shift = inst.rs2;
                        self.regs[inst.rd] = self.regs[inst.rs1] << shift;
                    },
                    0x2 => { // SLTI (set less than immediate)
                        let result = ((self.regs[inst.rs1] as i64) < imm as i64);
                        self.regs[inst.rd] = result as u64;
                    },
                    0x3 => { // STLIU
                        let result = ((self.regs[inst.rs1] as u64) < imm);
                        self.regs[inst.rd] = result as u64;
                    },
                    0x4 => { // XORI
                        self.regs[inst.rd] = self.regs[inst.rs1] ^ imm;
                    },
                    0x5 => {
                        match(inst.funct7) {
                            0x0 => { // SRLI (Logical shift right)
                                let shift = inst.rs2;
                                self.regs[inst.rd] = self.regs[inst.rs1] >> shift;
                            }
                            _ => { // SRAI (Arithmetic shift right)
                                let sign = self.regs[inst.rs1] & (1 << 31);
                                let shift = inst.rs2;
                                self.regs[inst.rd] = (self.regs[inst.rs1] >> shift) | sign;
                            }
                        } 
                    },
                    0x6 => { // ORI
                        self.regs[inst.rd] = self.regs[inst.rs1] | imm;
                    },
                    0x7 => { // ANDI
                        self.regs[inst.rd] = self.regs[inst.rs1] & imm;
                    }
                    _ => {}
                }
            }
            0x37 => { // LUI (load 12-31 bits into register)
                let imm = ((inst.funct7 as u64) << 13) | ((inst.rs2 as u64) << 8) | ((inst.rs1 as u64) << 3) | inst.funct3 as u64;
                self.regs[inst.rd] = (imm << 12);
                return Ok(self.pc + 4);
            }
            0x73 => { // Diff CSR instructions have the same OP code, but differing rs2/funct7
                match(inst.rs2, inst.funct7) {
                    (0x3, _) => { // csrrc
                        let csr = (inst.funct7 << 5) | inst.rs2;
                        let temp = self.csr.load(csr);
                        let csr_value = temp & !self.regs[inst.rs1];
                        self.csr.store(csr, csr_value);
                        self.regs[inst.rd] = temp;
                        return Ok(self.pc + 4);
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
                        return Ok(self.csr.load(csr::SEPC) & !0b11);
                    }
                    (_, _) => {
                        println!("CSR instruction not supported yet!");
                        return Ok(self.pc + 4);
                    }
                }
            }
            _ => {
                println!("Not implemented");
                println!("Op {} not implemented yet!", inst.opcode);
                return Ok(self.pc + 4);
            }

        }
        return Ok(self.pc + 4);
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use asm_riscv::{I, Reg};

    fn execute_instructions(cpu: &mut Cpu, instr: &[I]) {
        for inst in instr.iter() {
            let machine_code = u32::from(*inst);
            let d = cpu.decode(machine_code);

            cpu.execute(d);
        }
    }

    #[test]
    fn test_execute_lui() {
        let mut cpu = Cpu::new(Vec::new());

        let instr = [
            I::LUI { d: Reg::T3, im: 1 },   
            I::LUI { d: Reg::T4, im: 256 },
        ];
        execute_instructions(&mut cpu, &instr);

        assert_eq!(cpu.regs[28], 1<<12);
        assert_eq!(cpu.regs[29], 256<<12);
    }

    #[test]
    fn test_execute_add_addi() {
       let mut cpu = Cpu::new(Vec::new());
 
        let instr = [
            I::ADDI { d: Reg::T3, s: Reg::T3, im: 3 },
            I::ADDI { d: Reg::T4, s: Reg::T4, im: 4 },
            I::ADD { d: Reg::T5, s1: Reg::T3, s2: Reg::T4 }
        ];

        execute_instructions(&mut cpu, &instr);

        assert_eq!(cpu.regs[28], 3);
        assert_eq!(cpu.regs[29], 4);
        assert_eq!(cpu.regs[30], 7);
    }
    
    #[test]
    fn test_execute_slli_srli() {
       let mut cpu = Cpu::new(Vec::new());
 
        let instr = [
            I::ADDI { d: Reg::T4, s: Reg::T4, im: 2 },
            I::SLLI { d: Reg::T3, s: Reg::T4, im: 1 },
            I::SRLI { d: Reg::T5, s: Reg::T4, im: 1 },
        ];

        execute_instructions(&mut cpu, &instr);

        assert_eq!(cpu.regs[28], 4);
        assert_eq!(cpu.regs[30], 1);
    }

    #[test]
    fn test_execute_srai() {
       let mut cpu = Cpu::new(Vec::new());
 
        let instr = [
            I::ADDI { d: Reg::T4, s: Reg::T4, im: -2 },
            I::SRAI { d: Reg::T3, s: Reg::T4, im: 1 },
        ];

        execute_instructions(&mut cpu, &instr);

        assert_eq!(cpu.regs[28], 2047);
    }


    #[test]
    fn test_execute_andi_ori_xori() {
       let mut cpu = Cpu::new(Vec::new());
 
        let instr = [
            I::ADDI { d: Reg::T4, s: Reg::T4, im: 3 },
            I::ANDI { d: Reg::T3, s: Reg::T4, im: 5 },
            I::ORI { d: Reg::T5, s: Reg::T4, im: 4 },
            I::XORI { d: Reg::T6, s: Reg::T4, im: 3 },
        ];


        execute_instructions(&mut cpu, &instr);

        assert_eq!(cpu.regs[28], 1);
        assert_eq!(cpu.regs[30], 7);
        assert_eq!(cpu.regs[31], 0);
    }

    #[test]
    fn test_execute_stli_stliu() {
       let mut cpu = Cpu::new(Vec::new());
 
        let instr = [
            I::ADDI { d: Reg::T1, s: Reg::T1, im: 3 },
            I::SLTI { d: Reg::T2, s: Reg::T1, im: 1 },
            I::SLTI { d: Reg::T3, s: Reg::T1, im: 5 },
            I::SLTUI { d: Reg::T4, s: Reg::T1, im: 1 },
            I::SLTUI { d: Reg::T5, s: Reg::T1, im: 5 },
        ];

        execute_instructions(&mut cpu, &instr);

        assert_eq!(cpu.regs[6], 3);
        assert_eq!(cpu.regs[7], 0);
        assert_eq!(cpu.regs[28], 1);
        assert_eq!(cpu.regs[29], 0);
        assert_eq!(cpu.regs[30], 1);
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
