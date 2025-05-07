use std::fmt::Debug;

use crate::{compiler::arch::riscv::*, ir::IdentID};

pub type RV64Instruction<V = IdentID> = LinkerInstruction<RegRef<V>, V>;

#[derive(Copy, Clone)]
pub enum RegRef<V = IdentID, R = Reg> {
    Var(V),
    Reg(R),
}

impl<V: Debug, R: Debug> Debug for RegRef<V, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Var(v) => write!(f, "{{{:?}}}", v),
            Self::Reg(r) => r.fmt(f),
        }
    }
}
