#[derive(Debug)]
#[derive(PartialEq)]
pub struct R_Instr {
    pub opcode: u32,
    pub rd: usize,
    pub funct3: usize,
    pub rs1: usize,
    pub rs2: usize,
    pub funct7: usize,
}

