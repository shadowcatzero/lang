use crate::compiler::arch::riscv::Reg;

use super::VarID;

#[derive(Clone)]
pub struct IRAsmInstruction {
    op: String,
    args: Vec<RegRef>,
}

#[derive(Clone)]
pub enum RegRef {
    Var(VarID),
    Reg(String),
}

