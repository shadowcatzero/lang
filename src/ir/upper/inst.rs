use crate::{common::FileSpan, ir::VarID};
use std::fmt::Debug;

use super::IRUInstruction;

#[derive(Clone, Copy)]
pub struct VarInst {
    pub id: VarID,
    pub span: FileSpan,
}

pub struct IRUInstrInst {
    pub i: IRUInstruction,
    pub span: FileSpan,
}

impl Debug for VarInst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.id)
    }
}

impl Debug for IRUInstrInst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.i)
    }
}
