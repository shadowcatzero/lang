use super::{Type, UInstrInst, UInstruction};
use crate::{
    common::FileSpan,
    ir::{Len, ID},
};
use std::{collections::HashMap, fmt::Debug};

pub type NamePath = Vec<String>;

// "effective" (externally visible) kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KindTy {
    Type,
    Var,
    Struct,
    Fn,
}

impl KindTy {
    pub fn str(&self) -> &'static str {
        match self {
            KindTy::Type => "type",
            KindTy::Var => "variable",
            KindTy::Fn => "function",
            KindTy::Struct => "struct",
        }
    }
}

pub trait Kind {
    fn ty() -> KindTy;
}

impl Kind for UFunc {
    fn ty() -> KindTy {
        KindTy::Fn
    }
}

impl Kind for UVar {
    fn ty() -> KindTy {
        KindTy::Var
    }
}

impl Kind for UStruct {
    fn ty() -> KindTy {
        KindTy::Struct
    }
}

impl Kind for Type {
    fn ty() -> KindTy {
        KindTy::Type
    }
}

pub type FnID = ID<UFunc>;
pub type VarID = ID<UVar>;
pub type VarInstID = ID<VarInst>;
pub type TypeID = ID<Type>;
pub type GenericID = ID<UGeneric>;
pub type StructID = ID<UStruct>;
pub type DataID = ID<UData>;
pub type ModID = ID<UModule>;

#[derive(Clone)]
pub struct UFunc {
    pub name: String,
    pub origin: Origin,
    pub args: Vec<VarID>,
    pub gargs: Vec<GenericID>,
    pub ret: TypeID,
    pub instructions: Vec<UInstrInst>,
}

#[derive(Clone)]
pub struct StructField {
    pub ty: TypeID,
}

#[derive(Clone)]
pub struct UStruct {
    pub name: String,
    pub origin: Origin,
    pub fields: HashMap<String, StructField>,
    pub gargs: Vec<GenericID>,
}

#[derive(Clone)]
pub struct UGeneric {
    pub name: String,
    pub origin: Origin,
}

#[derive(Clone)]
pub struct UVar {
    pub name: String,
    pub origin: Origin,
    pub ty: TypeID,
    pub parent: Option<VarID>,
    pub children: Vec<VarID>,
}

/// these are more like "expressions", need to find good name
/// eg. a::b::c::<T,U>.d.e
#[derive(Clone, Debug)]
pub struct VarInst {
    pub status: VarStatus,
    pub origin: Origin,
}

#[derive(Clone, Debug)]
pub enum VarStatus {
    Var(VarID),
    Struct(StructID, Vec<TypeID>),
    Unres {
        path: ModPath,
        name: String,
        gargs: Vec<TypeID>,
        fields: Vec<MemberID>,
    },
    Partial {
        v: VarID,
        fields: Vec<MemberID>,
    },
    Cooked,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemberID {
    pub name: String,
    pub origin: Origin,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModPath {
    pub id: ModID,
    pub path: Vec<MemberID>,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct VarOffset {
    pub id: VarID,
    pub offset: Len,
}

#[derive(Clone)]
pub struct UData {
    pub name: String,
    pub ty: TypeID,
    pub content: Vec<u8>,
}

#[derive(Clone)]
pub struct UModule {
    pub name: String,
    pub members: HashMap<String, Member>,
    pub children: HashMap<String, ModID>,
    pub parent: Option<ModID>,
}

#[derive(Clone)]
pub struct Member {
    pub id: MemberTy,
    // pub visibility: Visibility
}

#[derive(Clone)]
pub enum MemberTy {
    Fn(FnID),
    Struct(StructID),
    Var(VarID),
}

pub type Origin = FileSpan;

impl UFunc {
    pub fn flat_iter(&self) -> impl Iterator<Item = &UInstrInst> {
        InstrIter::new(self.instructions.iter())
    }
}

pub struct InstrIter<'a> {
    iters: Vec<core::slice::Iter<'a, UInstrInst>>,
}

impl<'a> InstrIter<'a> {
    pub fn new(iter: core::slice::Iter<'a, UInstrInst>) -> Self {
        Self { iters: vec![iter] }
    }
}

impl<'a> Iterator for InstrIter<'a> {
    type Item = &'a UInstrInst;

    fn next(&mut self) -> Option<Self::Item> {
        let iter = self.iters.last_mut()?;
        let Some(next) = iter.next() else {
            self.iters.pop();
            return self.next();
        };
        match &next.i {
            UInstruction::Loop { body } => self.iters.push(body.iter()),
            UInstruction::If { cond: _, body } => self.iters.push(body.iter()),
            _ => (),
        }
        Some(next)
    }
}

impl VarInst {
    pub fn id(&self) -> Option<VarID> {
        match &self.status {
            VarStatus::Var(id) => Some(*id),
            VarStatus::Unres { .. } => None,
            VarStatus::Partial { .. } => None,
            VarStatus::Cooked => None,
        }
    }
}
