use super::*;
use crate::compiler::arch::riscv64::Reg;
use arch::riscv64::RV64Instruction;
use std::collections::HashMap;

#[derive(Debug)]
pub struct IRLFunction {
    pub name: String,
    pub instructions: Vec<IRLInstruction>,
    pub stack: HashMap<VarID, Size>,
    pub args: Vec<(VarID, Size)>,
    pub ret_size: Size,
    pub makes_call: bool,
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
        offset: Size,
        src: Symbol,
    },
    LoadData {
        dest: VarID,
        offset: Size,
        src: Symbol,
        len: Len,
    },
    Call {
        dest: Option<(VarID, Size)>,
        f: Symbol,
        args: Vec<(VarID, Size)>,
    },
    AsmBlock {
        instructions: Vec<RV64Instruction>,
        args: Vec<(Reg, VarID)>,
    },
    Ret {
        src: VarID,
    },
}
