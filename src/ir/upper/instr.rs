use std::collections::HashMap;

use super::{arch::riscv64::RV64Instruction, DataID, IdentID, Origin, ResStage, Unresolved};
use crate::compiler::arch::riscv::Reg;

pub enum UInstruction<S: ResStage = Unresolved> {
    Mv {
        dst: S::Var,
        src: S::Var,
    },
    Ref {
        dst: S::Var,
        src: S::Var,
    },
    Deref {
        dst: S::Var,
        src: S::Var,
    },
    LoadData {
        dst: S::Var,
        src: DataID,
    },
    LoadSlice {
        dst: S::Var,
        src: DataID,
    },
    Call {
        dst: S::Var,
        f: S::Func,
        args: Vec<S::Var>,
    },
    AsmBlock {
        instructions: Vec<RV64Instruction<S::Var>>,
        args: Vec<AsmBlockArg<S::Var>>,
    },
    Ret {
        src: S::Var,
    },
    Construct {
        dst: S::Var,
        struc: S::Struct,
        fields: HashMap<String, S::Var>,
    },
    If {
        cond: S::Var,
        body: Vec<UInstrInst<S>>,
    },
    Loop {
        body: Vec<UInstrInst<S>>,
    },
    Break,
    Continue,
}

pub struct UInstrInst<S: ResStage = Unresolved> {
    pub i: UInstruction<S>,
    pub origin: Origin,
}

#[derive(Debug, Clone)]
pub struct AsmBlockArg<V = IdentID> {
    pub var: V,
    pub reg: Reg,
    pub ty: AsmBlockArgType,
}

#[derive(Debug, Clone, Copy)]
pub enum AsmBlockArgType {
    In,
    Out,
}
