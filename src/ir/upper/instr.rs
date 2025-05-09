use std::collections::HashMap;

use super::{arch::riscv64::RV64Instruction, *};
use crate::compiler::arch::riscv::Reg;

pub trait ResStage {
    type Var;
    type Func;
    type Struct;
    type Type;
}

pub struct Unresolved;
impl ResStage for Unresolved {
    type Var = VarRes;
    type Func = IdentID;
    type Struct = IdentID;
    type Type = TypeRes;
}

pub struct Resolved;
impl ResStage for Resolved {
    type Var = VarID;
    type Func = FnInst;
    type Struct = StructInst;
    type Type = TypeID;
}

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
        body: Vec<InstrID>,
    },
    Loop {
        body: Vec<InstrID>,
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
