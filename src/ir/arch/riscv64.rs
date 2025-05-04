use crate::{compiler::arch::riscv::*, ir::UIdent};

pub type RV64Instruction = LinkerInstruction<RegRef, UIdent>;

#[derive(Copy, Clone)]
pub enum RegRef {
    Var(UIdent),
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

