use super::{Reg, OPCODE_MASK};
use crate::{compiler::program::Instr, util::{bit, bits, in_bit_range}};

pub struct Instruction(u32);

use Instruction as I;

impl Instr for Instruction {
    fn to_le_bytes(&self) -> impl IntoIterator<Item = u8> {
        self.0.to_le_bytes().into_iter()
    }
}

pub const fn r_type(funct7: u32, rs2: Reg, rs1: Reg, funct3: u32, rd: Reg, opcode: u32) -> I {
    I((funct7 << 25)
        + (rs2.val() << 20)
        + (rs1.val() << 15)
        + (funct3 << 12)
        + (rd.val() << 7)
        + opcode)
}
pub const fn i_type(imm: i32, rs1: Reg, funct3: u32, rd: Reg, opcode: u32) -> I {
    debug_assert!(in_bit_range(imm, 11, 0));
    I((bits(imm, 11, 0) << 20) + (rs1.val() << 15) + (funct3 << 12) + (rd.val() << 7) + opcode)
}
pub const fn s_type(rs2: Reg, rs1: Reg, funct3: u32, imm: i32, opcode: u32) -> I {
    debug_assert!(in_bit_range(imm, 11, 0));
    I((bits(imm, 11, 5) << 25)
        + (rs2.val() << 20)
        + (rs1.val() << 15)
        + (funct3 << 12)
        + (bits(imm, 4, 0) << 7)
        + opcode)
}
pub const fn b_type(rs2: Reg, rs1: Reg, funct3: u32, imm: i32, opcode: u32) -> I {
    debug_assert!(in_bit_range(imm, 12, 1));
    I((bit(imm, 12) << 31)
        + (bits(imm, 10, 5) << 25)
        + (rs2.val() << 20)
        + (rs1.val() << 15)
        + (funct3 << 8)
        + (bits(imm, 4, 1) << 8)
        + (bit(imm, 11) << 7)
        + opcode)
}
pub const fn u_type(imm: i32, rd: Reg, opcode: u32) -> I {
    debug_assert!(in_bit_range(imm, 31, 12));
    I((bits(imm, 31, 12) << 12) + (rd.val() << 7) + opcode)
}
pub const fn j_type(imm: i32, rd: Reg, opcode: u32) -> I {
    debug_assert!(in_bit_range(imm, 20, 1));
    I((bit(imm, 20) << 31)
        + (bits(imm, 10, 1) << 21)
        + (bit(imm, 11) << 20)
        + (bits(imm, 19, 12) << 12)
        + (rd.val() << 7)
        + opcode)
}
