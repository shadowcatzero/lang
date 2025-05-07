use super::*;
use crate::{compiler::arch::riscv::Reg, ir::arch::riscv64::RegRef};
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
        dst: VarID,
        dst_offset: Size,
        src: VarID,
        src_offset: Size,
    },
    Ref {
        dst: VarID,
        src: VarID,
    },
    LoadAddr {
        dst: VarID,
        offset: Size,
        src: Symbol,
    },
    LoadData {
        dst: VarID,
        offset: Size,
        src: Symbol,
        len: Len,
    },
    Call {
        dst: Option<(VarID, Size)>,
        f: Symbol,
        args: Vec<(VarID, Size)>,
    },
    AsmBlock {
        instructions: Vec<RV64Instruction<VarID>>,
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
