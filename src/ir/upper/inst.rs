use crate::ir::VarID;
use std::fmt::Debug;

use super::{Origin, UInstruction};

#[derive(Clone, Copy)]
pub struct VarInst {
    pub id: VarID,
    pub origin: Origin,
}

#[derive(Clone)]
pub struct UInstrInst {
    pub i: UInstruction,
    pub origin: Origin,
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
