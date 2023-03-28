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
/*
enum Instruction {
    R_Type(Opcode, Register, int, Register, Register, int),
    I_Type(),
    S_Type(),
    U_Type(),
}



impl R_Instr {
    pub fn from_assembly_string(str: String) -> R_Instr {
        let array = str.split(" ");
        let inst = array[0];
    }
}
*/
