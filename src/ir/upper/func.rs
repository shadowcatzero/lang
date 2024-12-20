use std::fmt::Write;

use super::{arch::riscv64::RV64Instruction, DataID, FnID, Len, VarID};
use crate::{compiler::arch::riscv64::Reg, util::Padder};

pub struct IRUFunction {
    pub name: String,
    pub args: Vec<VarID>,
    pub instructions: Vec<IRUInstruction>,
}

pub enum IRUInstruction {
    Mv {
        dest: VarID,
        src: VarID,
    },
    Ref {
        dest: VarID,
        src: VarID,
    },
    LoadData {
        dest: VarID,
        src: DataID,
    },
    LoadSlice {
        dest: VarID,
        src: DataID,
        len: Len,
    },
    LoadFn {
        dest: VarID,
        src: FnID,
    },
    Call {
        dest: VarID,
        f: VarID,
        args: Vec<VarID>,
    },
    AsmBlock {
        instructions: Vec<RV64Instruction>,
        args: Vec<(Reg, VarID)>,
    },
    Ret {
        src: VarID,
    },
}

pub struct IRInstructions {
    vec: Vec<IRUInstruction>,
}

impl IRUFunction {
    pub fn new(name: String, args: Vec<VarID>, instructions: IRInstructions) -> Self {
        Self {
            name,
            args,
            instructions: instructions.vec,
        }
    }
}

impl IRInstructions {
    pub fn new() -> Self {
        Self { vec: Vec::new() }
    }
    pub fn push(&mut self, i: IRUInstruction) {
        self.vec.push(i);
    }
}

impl std::fmt::Debug for IRUInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mv { dest, src } => write!(f, "{dest:?} <- {src:?}"),
            Self::Ref { dest, src } => write!(f, "{dest:?} <- &{src:?}"),
            Self::LoadData { dest, src } => write!(f, "{dest:?} <- {src:?}"),
            Self::LoadFn { dest, src } => write!(f, "{dest:?} <- {src:?}"),
            Self::LoadSlice { dest, src, len } => write!(f, "{dest:?} <- &[{src:?}; {len}]"),
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
