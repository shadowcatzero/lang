use std::fmt::Write;

use super::{arch::riscv64::RV64Instruction, inst::VarInst, DataID, FnID, IRUInstrInst, Type, VarID};
use crate::{common::FileSpan, compiler::arch::riscv::Reg, util::Padder};

pub struct IRUFunction {
    pub name: String,
    pub args: Vec<VarID>,
    pub ret: Type,
    pub instructions: Vec<IRUInstrInst>,
}

pub enum IRUInstruction {
    Mv {
        dest: VarInst,
        src: VarInst,
    },
    Ref {
        dest: VarInst,
        src: VarInst,
    },
    LoadData {
        dest: VarInst,
        src: DataID,
    },
    LoadSlice {
        dest: VarInst,
        src: DataID,
    },
    LoadFn {
        dest: VarInst,
        src: FnID,
    },
    Call {
        dest: VarInst,
        f: VarInst,
        args: Vec<VarInst>,
    },
    AsmBlock {
        instructions: Vec<RV64Instruction>,
        args: Vec<(Reg, VarInst)>,
    },
    Ret {
        src: VarInst,
    },
}

pub struct IRInstructions {
    vec: Vec<IRUInstrInst>,
}

impl IRUFunction {
    pub fn new(name: String, args: Vec<VarID>, ret: Type, instructions: IRInstructions) -> Self {
        Self {
            name,
            ret,
            args,
            instructions: instructions.vec,
        }
    }
}

impl IRInstructions {
    pub fn new() -> Self {
        Self { vec: Vec::new() }
    }
    pub fn push(&mut self, i: IRUInstruction, span: FileSpan) {
        self.vec.push(IRUInstrInst { i, span });
    }
}

impl std::fmt::Debug for IRUInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mv { dest, src } => write!(f, "{dest:?} <- {src:?}"),
            Self::Ref { dest, src } => write!(f, "{dest:?} <- &{src:?}"),
            Self::LoadData { dest, src } => write!(f, "{dest:?} <- {src:?}"),
            Self::LoadFn { dest, src } => write!(f, "{dest:?} <- {src:?}"),
            Self::LoadSlice { dest, src } => write!(f, "{dest:?} <- &[{src:?}]"),
            Self::Call {
                dest,
                f: func,
                args,
            } => write!(f, "{dest:?} <- {func:?}({args:?})"),
            Self::AsmBlock { args, instructions } => write!(f, "asm {args:?} {instructions:#?}"),
            Self::Ret { src } => f.debug_struct("Ret").field("src", src).finish(),
        }
    }
}

impl std::fmt::Debug for IRUFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{:?}", &self.name, self.args)?;
        if !self.instructions.is_empty() {
            f.write_str("{\n    ")?;
            let mut padder = Padder::new(f);
            for i in &self.instructions {
                // they don't expose wrap_buf :grief:
                padder.write_str(&format!("{i:?};\n"))?;
            }
            f.write_char('}')?;
        } else {
            f.write_str("{}")?;
        }
        Ok(())
    }
}
