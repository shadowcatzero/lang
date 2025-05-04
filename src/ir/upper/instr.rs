use std::{collections::HashMap, fmt::Write};

use super::{arch::riscv64::RV64Instruction, DataID, FnID, Origin, UFunc, VarInst, VarInstID};
use crate::{compiler::arch::riscv::Reg, util::Padder};

#[derive(Clone)]
pub enum UInstruction {
    Mv {
        dst: VarInstID,
        src: VarInstID,
    },
    Ref {
        dst: VarInstID,
        src: VarInstID,
    },
    Deref {
        dst: VarInstID,
        src: VarInstID,
    },
    LoadData {
        dst: VarInstID,
        src: DataID,
    },
    LoadSlice {
        dst: VarInstID,
        src: DataID,
    },
    LoadFn {
        dst: VarInstID,
        src: FnID,
    },
    Call {
        dst: VarInstID,
        f: VarInstID,
        args: Vec<VarInstID>,
    },
    AsmBlock {
        instructions: Vec<RV64Instruction>,
        args: Vec<AsmBlockArg>,
    },
    Ret {
        src: VarInstID,
    },
    Construct {
        dst: VarInstID,
        struc: VarInstID,
        fields: HashMap<String, VarInstID>,
    },
    If {
        cond: VarInstID,
        body: Vec<UInstrInst>,
    },
    Loop {
        body: Vec<UInstrInst>,
    },
    Break,
    Continue,
}

#[derive(Clone)]
pub struct UInstrInst {
    pub i: UInstruction,
    pub origin: Origin,
}

impl std::fmt::Debug for UInstrInst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.i)
    }
}

#[derive(Debug, Clone)]
pub struct AsmBlockArg {
    pub var: VarInstID,
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
            Self::Construct { dst: dest, struc, fields } => write!(f, "{dest:?} <- {struc:?}{fields:?}")?,
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
