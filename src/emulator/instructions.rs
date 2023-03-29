/*
+---------------------+-----+-----+--------+-----------------+--------+
|       funct7        | rs2 | rs1 | funct3 |        rd       | opcode | R-type
+---------------------------+-----+--------+-----------------+--------+
|         imm[11:0]         | rs1 | funct3 |        rd       | opcode | I-type
+---------------------+-----+-----+--------+-----------------+--------+
|      imm[11:5]      | rs2 | rs1 | funct3 |     imm[4:0]    | opcode | S-type
+---------+-----------+-----+-----+--------+-----------------+--------+
| imm[12] | imm[10:5] | rs2 | rs1 | funct3 | imm[4:1]imm[11] | opcode | B-type
+---------+-----------+-----+-----+--------+-----------------+--=-----+
|           imm[31:12]                     |        rd       | opcode | U-type
+---------+-----------+-------+------------+-----------------+--------+
| imm[20] | imm[10:1] |imm[11]| imm[19:12] |        rd       | opcode | J-type
+---------+-----------+--------+-----------+-----------------+--------+
*/

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

impl R_Instr {
    pub fn from_u32(inst: u32) -> R_Instr {
        let op: u32 = inst & 0x7f; // Note: 0x7f -> 0b0111_1111 , which acts as a mask
        let rd = ((inst >> 7) & 0x1f) as usize;
        let funct3 = ((inst >> 12) & 0x7) as usize;
        let rs1 = ((inst >> 15) & 0x1f) as usize;
        let rs2 = ((inst >> 20) & 0x1f) as usize;
        let funct7 = ((inst >> 25) & 0x7f) as usize;
        return R_Instr { opcode: op, rd: rd, funct3: funct3, rs1: rs1, rs2: rs2, funct7: funct7 };
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct I_Instr {
    pub opcode: u32,
    pub rd: usize,
    pub funct3: usize,
    pub rs1: usize,
    pub imm: usize,
}

impl I_Instr {
    pub fn from_u32(inst: u32) -> I_Instr {
        let op: u32 = inst & 0x7f;
        let rd = ((inst >> 7) & 0x1f) as usize;
        let funct3 = ((inst >> 12) & 0x7) as usize;
        let rs1 = ((inst >> 15) & 0x1f) as usize;
        let imm = ((inst >> 20) & 0xfff) as usize;
        return I_Instr { opcode: op, rd: rd, funct3: funct3, rs1: rs1, imm: imm };
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct S_Instr {
    pub opcode: u32,
    pub funct3: usize,
    pub rs1: usize,
    pub rs2: usize,
    pub imm: usize,
}

impl S_Instr {
    pub fn from_u32(inst: u32) -> S_Instr {
        let op: u32 = inst & 0x7f;
        let imm1 = ((inst >> 7) & 0x1f) as usize;
        let funct3 = ((inst >> 12) & 0x7) as usize;
        let rs1 = ((inst >> 15) & 0x1f) as usize;
        let rs2 = ((inst >> 20) & 0x1f) as usize;
        let imm2 = ((inst >> 25) & 0x7f) as usize;
        let imm = (imm2 << 5) | imm1;
        return S_Instr { opcode: op, funct3: funct3, rs1: rs1, rs2: rs2, imm: imm };
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct B_Instr {
    pub opcode: u32,
    pub funct3: usize,
    pub rs1: usize,
    pub rs2: usize,
    pub imm: usize,
}

impl B_Instr {
    pub fn from_u32(inst: u32) -> B_Instr {
        let op: u32 = inst & 0x7f;
        let imm3 = ((inst >> 7) & 0x1) as usize;
        let imm1 = ((inst >> 8) & 0xf) as usize;
        let imm2 = ((inst >> 25) & 0x3f) as usize;
        let imm4 = ((inst >> 31) & 0x1) as usize;
        let funct3 = ((inst >> 12) & 0x7) as usize;
        let rs1 = ((inst >> 15) & 0x1f) as usize;
        let rs2 = ((inst >> 20) & 0x1f) as usize;
        // Note: B-Type uses imm as a PC offset, so the last two bits should theoretically be 00
        // so the offset is 4-byte aligned. However, some extensions use 2-byte aligned
        // instructions, so B-Type assumes imm ends with 0 and does not encode it. For our
        // purposes, if imm ends with 10, then we will throw a fault on access later
        let imm = (imm4 << 12) | (imm3 << 11) | (imm2 << 5) | (imm1 << 1);
        return B_Instr { opcode: op, funct3: funct3, rs1: rs1, rs2: rs2, imm: imm };
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct U_Instr {
    pub opcode: u32,
    pub rd: usize,
    pub imm: usize,
}

impl U_Instr {
    pub fn from_u32(inst: u32) -> U_Instr {
        let op: u32 = inst & 0x7f;
        let rd = ((inst >> 7) & 0x1f) as usize;
        // Note: imm here is the most-significant 20 bits of 32 bits, and the rest is 0
        // This instruction is meant to compose with I type instructions to write 32 bits total
        let imm = (inst >> 12) as usize;
        return U_Instr { opcode: op, rd: rd, imm: imm };
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct J_Instr {
    pub opcode: u32,
    pub rd: usize,
    pub imm: usize,
}

impl J_Instr {
    pub fn from_u32(inst: u32) -> J_Instr {
        let op: u32 = inst & 0x7f;
        let rd = ((inst >> 7) & 0x1f) as usize;
        let imm4 = (inst >> 31) as usize;
        let imm1 = ((inst >> 21) & 0x3ff) as usize;
        let imm2 = ((inst >> 20) & 0x1) as usize;
        let imm3 = ((inst >> 12) & 0xff) as usize;
        let imm = (imm4 << 20) | (imm3 << 12) | (imm2 << 11) | (imm1 << 1); 
        return J_Instr { opcode: op, rd: rd, imm: imm };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_R_instr_decode() {
        // add x30, x28, x29
        let binary_add_instr = "00000001110111100000111100110011";
        // Alternates 1 and 0s on values to check index edge cases
        let binary_test = "00000001111100000111000001111111";

        let add_instr = R_Instr::from_u32(u32::from_str_radix(binary_add_instr, 2).unwrap());
        let test_instr = R_Instr::from_u32(u32::from_str_radix(binary_test, 2).unwrap());

        assert_eq!(R_Instr{ opcode: 0x33, funct3: 0x0, funct7: 0x0, rs1: 0x1c, rs2: 0x1d, rd: 0x1e}, add_instr);
        assert_eq!(R_Instr{ opcode: 0x7f, funct3: 0x7, funct7: 0x0, rs1: 0x0, rs2: 0x1f, rd: 0x0}, test_instr);
    }
    
    #[test]
    fn test_I_instr_decode() {
        // addi x30, x28, 55
        let binary_addi_instr = "00000011011111100000111100010011";
        // Alternates 1 and 0s on values to check index edge cases
        let binary_test = "11111111111100000111000001111111";

        let addi_instr = I_Instr::from_u32(u32::from_str_radix(binary_addi_instr, 2).unwrap());
        let test_instr = I_Instr::from_u32(u32::from_str_radix(binary_test, 2).unwrap());

        assert_eq!(I_Instr{ opcode: 0x13, funct3: 0x0, rs1: 0x1c, rd: 0x1e, imm: 0x037 }, addi_instr);
        assert_eq!(I_Instr{ opcode: 0x7f, funct3: 0x7, rs1: 0x0, rd: 0x0, imm: 0xfff }, test_instr);
    }

    #[test]
    fn test_S_instr_decode() {
        // sw x30, 32(x29)
        let binary_sw_instr = "00000011111011101010000000100011";
        // Alternates 1 and 0s on values to check index edge cases
        let binary_test = "00000001111100000111000001111111";

        let sw_instr = S_Instr::from_u32(u32::from_str_radix(binary_sw_instr, 2).unwrap());
        let test_instr = S_Instr::from_u32(u32::from_str_radix(binary_test, 2).unwrap());

        assert_eq!(S_Instr{ opcode: 0x23, funct3: 0x2, rs1: 0x1d, rs2: 0x1e, imm: 0x020 }, sw_instr);
        assert_eq!(S_Instr{ opcode: 0x7f, funct3: 0x7, rs1: 0x0, rs2: 0x1f, imm: 0x0 }, test_instr);
    }

    #[test]
    fn test_B_instr_decode() {
        // beq x29, x30, 54
        let binary_beq_instr = "00000011111011101000101101100011";
        // Alternates 1 and 0s on values to check index edge cases
        let binary_test = "00000001111100000111000001111111";
        // TODO: Add extra test case that checks the reconstruction of imm

        let beq_instr = B_Instr::from_u32(u32::from_str_radix(binary_beq_instr, 2).unwrap());
        let test_instr = B_Instr::from_u32(u32::from_str_radix(binary_test, 2).unwrap());

        assert_eq!(B_Instr{ opcode: 0x63, funct3: 0x0, rs1: 0x1d, rs2: 0x1e, imm: 0x036 }, beq_instr);
        assert_eq!(B_Instr{ opcode: 0x7f, funct3: 0x7, rs1: 0x0, rs2: 0x1f, imm: 0x0 }, test_instr);
    }

    #[test]
    fn test_U_instr_decode() {
        // lui x29, 1998848
        let binary_lui_instr = "00000000000111101000111010110111";
        // Alternates 1 and 0s on values to check index edge cases
        let binary_test = "11111111111111111111000001111111";

        let lui_instr = U_Instr::from_u32(u32::from_str_radix(binary_lui_instr, 2).unwrap());
        let test_instr = U_Instr::from_u32(u32::from_str_radix(binary_test, 2).unwrap());

        assert_eq!(U_Instr{ opcode: 0x37, rd: 0x1d, imm: 0x1e8 }, lui_instr);
        assert_eq!(U_Instr{ opcode: 0x7f, rd: 0x0, imm: 0xfffff }, test_instr);
    }
    
    #[test]
    fn test_J_instr_decode() {
        // jal x29, 254
        let binary_jal_instr = "00001111111000000000111011101111";
        // Alternates 1 and 0s on values to check index edge cases
        let binary_test = "11111111111111111111000001111111";
        // TODO: Add extra test case that checks reconstruction of imm

        let jal_instr = J_Instr::from_u32(u32::from_str_radix(binary_jal_instr, 2).unwrap());
        let test_instr = J_Instr::from_u32(u32::from_str_radix(binary_test, 2).unwrap());

        assert_eq!(J_Instr{ opcode: 0x6f, rd: 0x1d, imm: 0x0fe }, jal_instr);
        assert_eq!(J_Instr{ opcode: 0x7f, rd: 0x0, imm: 0x1ffffe }, test_instr);
    }
}

