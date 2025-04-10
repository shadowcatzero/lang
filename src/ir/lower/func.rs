use super::*;
use crate::compiler::arch::riscv::Reg;
use arch::riscv64::RV64Instruction;
use std::collections::HashMap;

#[derive(Debug)]
pub struct IRLFunction {
    pub instructions: Vec<LInstruction>,
    pub stack: HashMap<VarID, Size>,
    pub subvar_map: HashMap<VarID, VarOffset>,
    pub args: Vec<(VarID, Size)>,
    pub ret_size: Size,
    pub makes_call: bool,
}

#[derive(Debug)]
pub enum LInstruction {
    Mv {
        dest: VarID,
        dest_offset: Size,
        src: VarID,
        src_offset: Size,
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
        inputs: Vec<(Reg, VarID)>,
        outputs: Vec<(Reg, VarID)>,
    },
    Ret {
        src: Option<VarID>,
    },
    // TODO I feel like this should be turned into control flow instructions, maybe...
    // not sure but LLVM has them so might be right play; seems optimal for optimization
    Jump(Symbol),
    Branch {
        to: Symbol,
        cond: VarID,
    },
    Mark(Symbol),
}

impl LInstruction {
    pub fn is_ret(&self) -> bool {
        matches!(self, Self::Ret { .. })
    }
}
