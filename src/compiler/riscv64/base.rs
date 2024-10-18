use crate::util::Bits32;

use super::Reg;

pub struct Instruction(u32);

use Instruction as I;

impl Instruction {
    pub fn to_le_bytes(&self) -> impl IntoIterator<Item = u8> {
        self.0.to_le_bytes().into_iter()
    }
    pub fn to_be_bytes(&self) -> impl IntoIterator<Item = u8> {
        self.0.to_be_bytes().into_iter()
    }
}

pub type Funct3 = Bits32<2, 0>;

pub const fn r_type(
    funct7: Bits32<6, 0>,
    rs2: Reg,
    rs1: Reg,
    funct3: Bits32<2, 0>,
    rd: Reg,
    opcode: u32,
) -> I {
    I((funct7.val() << 25)
        + (rs2.val() << 20)
        + (rs1.val() << 15)
        + (funct3.val() << 12)
        + (rd.val() << 7)
        + opcode)
}
pub const fn i_type(imm: Bits32<11, 0>, rs1: Reg, funct: Funct3, rd: Reg, opcode: u32) -> I {
    I((imm.val() << 20) + (rs1.val() << 15) + (funct.val() << 12) + (rd.val() << 7) + opcode)
}
pub const fn s_type(
    rs2: Reg,
    rs1: Reg,
    funct3: Funct3,
    imm: Bits32<11, 0>,
    opcode: u32,
) -> I {
    I((imm.bits(11, 5) << 25)
        + (rs2.val() << 20)
        + (rs1.val() << 15)
        + (funct3.val() << 12)
        + (imm.bits(4, 0) << 7)
        + opcode)
}
pub const fn b_type(
    rs2: Reg,
    rs1: Reg,
    funct3: Funct3,
    imm: Bits32<12, 1>,
    opcode: u32,
) -> I {
    I((imm.bit(12) << 31)
        + (imm.bits(10, 5) << 25)
        + (rs2.val() << 20)
        + (rs1.val() << 15)
        + (funct3.val() << 8)
        + (imm.bits(4, 1) << 8)
        + (imm.bit(11) << 7)
        + opcode)
}
pub const fn u_type(imm: Bits32<31, 12>, rd: Reg, opcode: u32) -> I {
    I((imm.bits(31, 12) << 12) + (rd.val() << 7) + opcode)
}
pub const fn j_type(imm: Bits32<20, 1>, rd: Reg, opcode: u32) -> I {
    I((imm.bit(20) << 31)
        + (imm.bits(10, 1) << 21)
        + (imm.bit(11) << 20)
        + (imm.bits(19, 12) << 12)
        + (rd.val() << 7)
        + opcode)
}
