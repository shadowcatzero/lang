use super::{Type, UInstrInst, UInstruction, UProgram};
use crate::{
    common::FileSpan,
    ir::{Len, ID},
};
use std::{collections::HashMap, fmt::Debug};

pub type NamePath = Vec<String>;

#[derive(Clone)]
pub struct UFunc {
    pub name: String,
    pub origin: Origin,
    pub args: Vec<VarID>,
    pub argtys: Vec<TypeID>,
    pub ret: TypeID,
    pub instructions: Vec<UInstrInst>,
}

#[derive(Clone)]
pub struct StructField {
    pub ty: Type,
}

#[derive(Clone)]
pub struct UStruct {
    pub name: String,
    pub origin: Origin,
    pub fields: HashMap<String, StructField>,
    pub generics: Vec<GenericID>,
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
    pub res: UVarTy,
}

#[derive(Clone)]
pub struct VarParent {
    id: VarID,
    path: Vec<String>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct MemberID {
    pub name: String,
    pub origin: Origin,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ModPath {
    pub m: ModID,
    pub path: Vec<MemberID>,
}

#[derive(Clone)]
pub enum UVarTy {
    Ptr(VarID),
    /// fully resolved & typed
    Res {
        parent: Option<VarParent>,
    },
    /// module doesn't exist yet
    Unres {
        path: ModPath,
        fields: Vec<MemberID>,
    },
    /// parent var exists but not typed enough for this field path
    Partial {
        v: VarID,
        fields: Vec<MemberID>,
    },
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct VarOffset {
    pub id: VarID,
    pub offset: Len,
}

#[derive(Clone)]
pub struct UData {
    pub name: String,
    pub ty: Type,
    pub content: Vec<u8>,
}

#[derive(Clone)]
pub struct UModule {
    pub name: String,
    pub members: HashMap<String, MemberID>,
    pub children: HashMap<String, ModID>,
    pub parent: Option<ModID>,
}

pub struct ModMissing {
    pub import_all: Vec<ModID>,
    pub vars: Vec<VarID>,
}

#[derive(Clone)]
pub enum Member {
    Fn(FnID),
    Struct(StructID),
    Var(VarID),
}

pub type Origin = FileSpan;

impl UFunc {
    pub fn ty(&self, program: &UProgram) -> Type {
        Type::Fn {
            args: self.argtys.clone(),
            ret: Box::new(self.ret.clone()),
        }
    }
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

pub const NAMED_KINDS: usize = 5;

pub type FnID = ID<UFunc>;
pub type VarID = ID<UVar>;
pub type TypeID = ID<Type>;
pub type GenericID = ID<UGeneric>;
pub type StructID = ID<UStruct>;
pub type DataID = ID<UData>;
pub type ModID = ID<Option<UModule>>;
