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

pub const fn load(width: Funct3, dest: Reg, offset: BitsI32<11, 0>, base: Reg) -> I {
    i_type(offset.to_u(), base, width, dest, LOAD)
}

pub const fn store(width: Funct3, src: Reg, offset: BitsI32<11, 0>, base: Reg) -> I {
    s_type(src, base, width, offset.to_u(), STORE)
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
