use super::*;
use crate::compiler::arch::riscv64::Reg;
use arch::riscv64::RV64Instruction;
use std::collections::HashMap;

#[derive(Debug)]
pub struct IRLFunction {
    pub name: String,
    pub instructions: Vec<IRLInstruction>,
    pub stack: HashMap<VarID, usize>,
    pub args: Vec<(VarID, usize)>,
}

#[derive(Debug)]
pub enum IRLInstruction {
    Mv {
        dest: VarID,
        src: VarID,
    },
    Ref {
        dest: VarID,
        src: VarID,
    },
    LoadAddr {
        dest: VarID,
        src: Symbol,
    },
    Call {
        dest: VarID,
        f: Symbol,
        args: Vec<(VarID, usize)>,
    },
    AsmBlock {
        instructions: Vec<RV64Instruction>,
        args: Vec<(Reg, VarID)>,
    },
    Ret {
        src: VarID,
    },
}
