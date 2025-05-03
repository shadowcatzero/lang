use crate::ir::VarID;
use std::fmt::Debug;

use super::{MemberID, ModPath, Origin, UInstruction};

#[derive(Clone)]
pub struct VarInst {
    pub status: VarStatus,
    pub origin: Origin,
}

#[derive(Clone)]
pub enum VarStatus {
    Res(VarID),
    Unres { mid: ModPath, fields: Vec<MemberID> },
    Partial { v: VarID, fields: Vec<MemberID> },
}

#[derive(Clone)]
pub struct VarParent {
    id: VarID,
    path: Vec<String>,
}

#[derive(Clone)]
pub struct UInstrInst {
    pub i: UInstruction,
    pub origin: Origin,
}

impl Debug for UInstrInst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.i)
    }
}
