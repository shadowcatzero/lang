use crate::util::{Bits32, BitsI32};

use super::*;

use RawInstruction as I;

pub const fn ecall() -> I {
    i_type(Bits32::new(0), zero, Bits32::new(0), zero, SYSTEM)
}
pub const fn ebreak() -> I {
    i_type(Bits32::new(1), zero, Bits32::new(0), zero, SYSTEM)
}
pub const fn auipc(dest: Reg, imm: BitsI32<31, 12>) -> I {
    u_type(imm.to_u(), dest, AUIPC)
}

pub const fn load(width: Width, dest: Reg, offset: BitsI32<11, 0>, base: Reg) -> I {
    i_type(offset.to_u(), base, width.code(), dest, LOAD)
}

pub const fn store(width: Width, src: Reg, offset: BitsI32<11, 0>, base: Reg) -> I {
    s_type(src, base, width.code(), offset.to_u(), STORE)
}

pub const fn opr(op: Op, dest: Reg, src1: Reg, src2: Reg) -> I {
    r_type(Bits32::new(0), src2, src1, op.code(), dest, OP)
}
pub const fn oprf7(op: Op, funct: Funct7, dest: Reg, src1: Reg, src2: Reg) -> I {
    r_type(funct, src2, src1, op.code(), dest, OP)
}
pub const fn opi(op: Op, dest: Reg, src: Reg, imm: BitsI32<11, 0>) -> I {
    i_type(imm.to_u(), src, op.code(), dest, IMM_OP)
}
pub const fn opif7(op: Op, funct: Funct7, dest: Reg, src: Reg, imm: BitsI32<4, 0>) -> I {
    i_type(
        Bits32::new(imm.to_u().val() + (funct.val() << 5)),
        src,
        op.code(),
        dest,
        IMM_OP,
    )
}
pub const fn jal(dest: Reg, offset: BitsI32<20, 1>) -> I {
    j_type(offset.to_u(), dest, JAL)
}
pub const fn jalr(dest: Reg, offset: BitsI32<11, 0>, base: Reg) -> I {
    i_type(offset.to_u(), base, Bits32::new(0), dest, JALR)
}

// pseudoinstructions that map to a single instruction

pub const fn j(offset: BitsI32<20, 1>) -> I {
    jal(zero, offset)
}
pub const fn ret() -> I {
    jalr(zero, BitsI32::new(0), ra)
}
