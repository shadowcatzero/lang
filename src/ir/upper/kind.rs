//! all main IR Upper data structures stored in UProgram

use super::{FnInst, ResErr, StructInst, Type, UInstrInst, UInstruction, UProgram};
use crate::{
    common::FileSpan,
    ir::{Len, ID},
};
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

pub trait ResStage {
    type Var;
    type Func;
    type Struct;
    type Type;
}

pub struct Unresolved;
impl ResStage for Unresolved {
    type Var = IdentID;
    type Func = IdentID;
    type Struct = IdentID;
    type Type = IdentID;
}

pub struct Resolved;
impl ResStage for Resolved {
    type Var = VarID;
    type Func = FnInst;
    type Struct = StructInst;
    type Type = TypeID;
}

pub type NamePath = Vec<String>;

pub type FnID = ID<UFunc>;
pub type VarID = ID<UVar>;
pub type IdentID = ID<UIdent>;
pub type TypeID = ID<Type>;
pub type GenericID = ID<UGeneric>;
pub type StructID = ID<UStruct>;
pub type DataID = ID<UData>;
pub type ModID = ID<UModule>;

pub struct UFunc {
    pub name: String,
    pub origin: Origin,
    pub args: Vec<VarID>,
    pub gargs: Vec<GenericID>,
    pub ret: TypeID,
    pub instructions: Vec<UInstrInst>,
}

pub struct StructField {
    pub ty: TypeID,
    pub origin: Origin,
    // pub vis: Visibility
}

pub struct UStruct {
    pub name: String,
    pub origin: Origin,
    pub fields: HashMap<String, StructField>,
    pub gargs: Vec<GenericID>,
}

pub struct UGeneric {
    pub name: String,
    pub origin: Origin,
}

pub struct UVar {
    pub name: String,
    pub origin: Origin,
    pub ty: TypeID,
    pub parent: Option<VarID>,
    pub children: HashMap<String, VarID>,
}

/// a generic identifier for all (identifiable) kinds
/// eg. a::b::c.d.e
/// or a::Result<T,_>
pub struct UIdent {
    pub status: IdentStatus,
    pub origin: Origin,
}

pub enum IdentStatus {
    Res(Res),
    Unres {
        base: ResBase,
        path: Vec<MemberIdent>,
    },
    Failed(Option<ResErr>),
    Cooked,
}

pub struct MemberIdent {
    pub ty: MemberTy,
    pub name: String,
    pub gargs: Vec<TypeID>,
    pub origin: Origin,
}

#[derive(Clone, Copy)]
pub enum MemberTy {
    Member,
    Field,
}

impl MemberTy {
    pub fn sep(&self) -> &'static str {
        match self {
            MemberTy::Member => "::",
            MemberTy::Field => ".",
        }
    }
}

impl Display for MemberTy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            MemberTy::Member => "member",
            MemberTy::Field => "field",
        })
    }
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
    pub parent: Option<ModID>,
    pub func: FnID,
}

#[derive(Clone)]
pub struct Member {
    pub id: MemberID,
    // pub visibility: Visibility
}

#[derive(Clone)]
pub enum MemberID {
    Fn(FnID),
    Struct(StructID),
    Var(VarID),
    Module(ModID),
    Type(TypeDef),
}

#[derive(Clone)]
pub struct TypeDef {
    pub gargs: Vec<GenericID>,
    pub ty: TypeID,
}

impl MemberID {
    pub fn kind(&self) -> KindTy {
        match self {
            MemberID::Fn(_) => KindTy::Fn,
            MemberID::Struct(_) => KindTy::Struct,
            MemberID::Var(_) => KindTy::Var,
            MemberID::Module(_) => KindTy::Module,
            MemberID::Type(_) => KindTy::Type,
        }
    }
    pub fn display_str(&self, p: &UProgram) -> String {
        let name = match self {
            MemberID::Var(id) => &p.vars[id].name,
            MemberID::Fn(id) => &p.fns[id].name,
            MemberID::Struct(id) => &p.structs[id].name,
            MemberID::Module(id) => &p.modules[id].name,
            MemberID::Type(id) => &p.type_name(id),
        };
        format!("{} '{}'", self.kind(), name)
    }
}

pub type Origin = FileSpan;

// "effective" (externally visible) kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KindTy {
    Type,
    Var,
    Struct,
    Fn,
    Module,
    Generic,
}

impl Display for KindTy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            KindTy::Type => "type",
            KindTy::Var => "variable",
            KindTy::Fn => "function",
            KindTy::Struct => "struct",
            KindTy::Module => "module",
            KindTy::Generic => "generic",
        })
    }
}

#[derive(Debug, Clone)]
pub enum Res {
    Var(VarID),
    Fn(FnInst),
    Struct(StructInst),
    Type(TypeID),
    Generic(GenericID),
    Module(ModID),
}

impl Res {
    pub fn kind(&self) -> KindTy {
        match self {
            Res::Var(..) => KindTy::Var,
            Res::Fn(..) => KindTy::Fn,
            Res::Struct(..) => KindTy::Struct,
            Res::Type(..) => KindTy::Type,
            Res::Module(..) => KindTy::Module,
            Res::Generic(..) => KindTy::Generic,
        }
    }

    pub fn display_str(&self, p: &UProgram) -> String {
        let name = match self {
            Res::Var(id) => &p.vars[id].name,
            Res::Fn(fi) => &p.fns[fi.id].name,
            Res::Struct(si) => &p.structs[si.id].name,
            Res::Type(id) => &p.type_name(id),
            Res::Generic(id) => &p.generics[id].name,
            Res::Module(id) => &p.modules[id].name,
        };
        format!("{} '{}'", self.kind(), name)
    }
}

#[derive(Clone)]
pub enum ResBase {
    Unvalidated(MemRes),
    Validated(Res),
}

impl ResBase {
    pub fn display_str(&self, p: &UProgram) -> String {
        match self {
            ResBase::Unvalidated(uv) => uv.display_str(p),
            ResBase::Validated(res) => res.display_str(p),
        }
    }
}

#[derive(Clone)]
pub struct MemRes {
    pub mem: Member,
    pub origin: Origin,
    pub gargs: Vec<TypeID>,
}

impl MemRes {
    pub fn display_str(&self, p: &UProgram) -> String {
        self.mem.id.display_str(p)
    }
}

impl IdentID {
    pub fn var(&self, p: &UProgram) -> Option<VarID> {
        match p.idents[self].status {
            IdentStatus::Res(Res::Var(id)) => Some(id),
            _ => None,
        }
    }
    pub fn fun<'a>(&self, p: &'a UProgram) -> Option<&'a FnInst> {
        match &p.idents[self].status {
            IdentStatus::Res(Res::Fn(i)) => Some(&i),
            _ => None,
        }
    }
    pub fn struc<'a>(&self, p: &'a UProgram) -> Option<&'a StructInst> {
        match &p.idents[self].status {
            IdentStatus::Res(Res::Struct(i)) => Some(&i),
            _ => None,
        }
    }
}

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
