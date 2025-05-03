use std::{collections::HashMap, fmt::Write};

use super::{arch::riscv64::RV64Instruction, inst::VarInst, DataID, FnID, UFunc, UInstrInst};
use crate::{compiler::arch::riscv::Reg, util::Padder};

#[derive(Clone)]
pub enum UInstruction {
    Mv {
        dst: VarInst,
        src: VarInst,
    },
    Ref {
        dst: VarInst,
        src: VarInst,
    },
    Deref {
        dst: VarInst,
        src: VarInst,
    },
    LoadData {
        dst: VarInst,
        src: DataID,
    },
    LoadSlice {
        dst: VarInst,
        src: DataID,
    },
    LoadFn {
        dst: VarInst,
        src: FnID,
    },
    Call {
        dst: VarInst,
        f: VarInst,
        args: Vec<VarInst>,
    },
    AsmBlock {
        instructions: Vec<RV64Instruction>,
        args: Vec<AsmBlockArg>,
    },
    Ret {
        src: VarInst,
    },
    Construct {
        dst: VarInst,
        fields: HashMap<String, VarInst>,
    },
    If {
        cond: VarInst,
        body: Vec<UInstrInst>,
    },
    Loop {
        body: Vec<UInstrInst>,
    },
    Break,
    Continue,
}

#[derive(Debug, Clone)]
pub struct AsmBlockArg {
    pub var: VarInst,
    pub reg: Reg,
    pub ty: AsmBlockArgType,
}

#[derive(Debug, Clone)]
pub enum AsmBlockArgType {
    In,
    Out,
}

impl std::fmt::Debug for UInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mv { dst: dest, src } => write!(f, "{dest:?} <- {src:?}")?,
            Self::Ref { dst: dest, src } => write!(f, "{dest:?} <- {src:?}&")?,
            Self::Deref { dst: dest, src } => write!(f, "{dest:?} <- {src:?}^")?,
            Self::LoadData { dst: dest, src } => write!(f, "{dest:?} <- {src:?}")?,
            Self::LoadFn { dst: dest, src } => write!(f, "{dest:?} <- {src:?}")?,
            Self::LoadSlice { dst: dest, src } => write!(f, "{dest:?} <- &[{src:?}]")?,
            Self::Call {
                dst: dest,
                f: func,
                args,
            } => write!(f, "{dest:?} <- {func:?}({args:?})")?,
            Self::AsmBlock { args, instructions } => write!(f, "asm {args:?} {instructions:#?}")?,
            Self::Ret { src } => f.debug_struct("Ret").field("src", src).finish()?,
            Self::Construct { dst: dest, fields } => write!(f, "{dest:?} <- {fields:?}")?,
            Self::If { cond, body } => {
                write!(f, "if {cond:?}:")?;
                if !body.is_empty() {
                    f.write_str("{\n    ")?;
                    let mut padder = Padder::new(f);
                    for i in body {
                        // they don't expose wrap_buf :grief:
                        padder.write_str(&format!("{i:?};\n"))?;
                    }
                    f.write_char('}')?;
                } else {
                    f.write_str("{}")?;
                }
            }
            Self::Loop { body } => {
                write!(f, "loop:")?;
                if !body.is_empty() {
                    f.write_str("{\n    ")?;
                    let mut padder = Padder::new(f);
                    for i in body {
                        // they don't expose wrap_buf :grief:
                        padder.write_str(&format!("{i:?};\n"))?;
                    }
                    f.write_char('}')?;
                } else {
                    f.write_str("{}")?;
                }
            }
            Self::Break => write!(f, "break")?,
            Self::Continue => write!(f, "continue")?,
        }
        Ok(())
    }
}

impl std::fmt::Debug for UFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.args)?;
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
