use crate::{common::FileSpan, ir::VarID};
use std::fmt::Debug;

use super::UInstruction;

#[derive(Clone, Copy)]
pub struct VarInst {
    pub id: VarID,
    pub span: FileSpan,
}

#[derive(Clone)]
pub struct UInstrInst {
    pub i: UInstruction,
    pub span: FileSpan,
}

impl Debug for VarInst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.id)
    }
}

impl Debug for UInstrInst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.i)
    }
}
