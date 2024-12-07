use crate::{compiler::arch::riscv64::*, ir::VarID};

#[derive(Copy, Clone)]
pub enum RV64Instruction {
    Ecall,
    Li { dest: RegRef, imm: i64 },
    Mv { dest: RegRef, src: RegRef },
    La { dest: RegRef, src: VarID },
    Ld { dest: RegRef, offset: i64, base: RegRef },
}

#[derive(Copy, Clone)]
pub enum RegRef {
    Var(VarID),
    Reg(Reg),
}

impl std::fmt::Debug for RegRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Var(v) => write!(f, "{{{:?}}}", v),
            Self::Reg(r) => r.fmt(f),
        }
    }
}

impl std::fmt::Debug for RV64Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ecall => write!(f, "ecall"),
            Self::Li { dest, imm } => write!(f, "li {dest:?}, {imm:?}"),
            Self::Mv { dest, src } => write!(f, "mv {dest:?}, {src:?}"),
            Self::La { dest, src } => write!(f, "la {dest:?}, {src:?}"),
            Self::Ld { dest, offset, base } => write!(f, "ld {dest:?}, {offset}({base:?})"),
        }
    }
}
