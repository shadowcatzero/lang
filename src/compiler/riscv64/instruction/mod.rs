mod base;
mod reg;
mod opcode;
mod func;

use base::*;
pub use reg::*;
use Instruction as I;
use opcode::*;
use func::{op::*, width};

pub const fn ecall() -> I {
    i_type(0, zero, 0, zero, SYSTEM)
}
pub const fn ebreak() -> I {
    i_type(1, zero, 0, zero, SYSTEM)
}
pub const fn auipc(dest: Reg, imm: i32) -> I {
    u_type(imm, dest, AUIPC)
}
pub const fn ld(dest: Reg, offset: i32, base: Reg) -> I {
    i_type(offset, base, width::D, dest, LOAD)
}
pub const fn lw(dest: Reg, offset: i32, base: Reg) -> I {
    i_type(offset, base, width::W, dest, LOAD)
}
pub const fn lb(dest: Reg, offset: i32, base: Reg) -> I {
    i_type(offset, base, width::B, dest, LOAD)
}
pub const fn sb(src: Reg, offset: i32, base: Reg) -> I {
    s_type(src, base, width::B, offset, STORE)
}
pub const fn sw(src: Reg, offset: i32, base: Reg) -> I {
    s_type(src, base, width::W, offset, STORE)
}
pub const fn sd(src: Reg, offset: i32, base: Reg) -> I {
    s_type(src, base, width::D, offset, STORE)
}
pub const fn addi(dest: Reg, src: Reg, imm: i32) -> I {
    i_type(imm, src, ADD, dest, IMM_OP)
}
pub const fn jal(offset: i32, dest: Reg) -> I {
    j_type(offset, dest, JAL)
}

// pseudo instructions that map to a single instruction

pub const fn j(offset: i32) -> I {
    jal(offset, zero)
}
pub const fn mv(dest: Reg, src: Reg) -> I {
    addi(dest, src, 0)
}
