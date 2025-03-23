use crate::{
    compiler::arch::riscv::Reg,
    util::{Bits32, BitsI32},
};

pub struct RawInstruction(u32);

use RawInstruction as I;

impl RawInstruction {
    pub fn to_le_bytes(&self) -> impl IntoIterator<Item = u8> {
        self.0.to_le_bytes().into_iter()
    }
    pub fn to_be_bytes(&self) -> impl IntoIterator<Item = u8> {
        self.0.to_be_bytes().into_iter()
    }
}

pub const SYSTEM: u32 = 0b1110011;
pub const LOAD: u32 = 0b0000011;
pub const STORE: u32 = 0b0100011;
pub const AUIPC: u32 = 0b0010111;
pub const IMM_OP: u32 = 0b0010011;
pub const OP: u32 = 0b0110011;
pub const JAL: u32 = 0b1101111;
pub const JALR: u32 = 0b1100111;

pub type Funct3 = Bits32<2, 0>;
pub type Funct7 = Bits32<6, 0>;

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
pub const fn s_type(rs2: Reg, rs1: Reg, funct3: Funct3, imm: Bits32<11, 0>, opcode: u32) -> I {
    I((imm.bits(11, 5) << 25)
        + (rs2.val() << 20)
        + (rs1.val() << 15)
        + (funct3.val() << 12)
        + (imm.bits(4, 0) << 7)
        + opcode)
}
pub const fn b_type(rs2: Reg, rs1: Reg, funct3: Funct3, imm: Bits32<12, 1>, opcode: u32) -> I {
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

pub fn opr(op: Funct3, funct: Funct7, dest: Reg, src1: Reg, src2: Reg) -> I {
    r_type(funct, src2, src1, op.into(), dest, OP)
}
pub fn opi(op: Funct3, dest: Reg, src: Reg, imm: BitsI32<11, 0>) -> RawInstruction {
    i_type(imm.to_u(), src, op.into(), dest, IMM_OP)
}
pub fn opif7(op: Funct3, funct: Funct7, dest: Reg, src: Reg, imm: BitsI32<4, 0>) -> I {
    i_type(
        Bits32::new(imm.to_u().val() + (funct.val() << 5)),
        src,
        op.into(),
        dest,
        IMM_OP,
    )
}
