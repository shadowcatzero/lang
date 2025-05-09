//! all main IR Upper data structures stored in UProgram

use super::*;
use crate::{
    common::FileSpan,
    ir::{Len, ID},
};
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

pub type NamePath = Vec<String>;

pub type FnID = ID<UFunc>;
pub type VarID = ID<UVar>;
pub type IdentID = ID<UIdent>;
pub type TypeID = ID<Type>;
pub type GenericID = ID<UGeneric>;
pub type StructID = ID<UStruct>;
pub type DataID = ID<UData>;
pub type ModID = ID<UModule>;
pub type InstrID = ID<UInstrInst>;

pub type VarRes = URes<VarID>;
pub type TypeRes = URes<VarID>;

pub struct UFunc {
    pub name: String,
    pub origin: Origin,
    pub args: Vec<VarID>,
    pub gargs: Vec<GenericID>,
    pub ret: TypeRes,
    pub instructions: Vec<InstrID>,
}

pub struct StructField {
    pub ty: TypeRes,
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
    pub ty: TypeRes,
    pub parent: Option<VarID>,
    pub children: HashMap<String, VarID>,
}

pub enum VarTy {
    Ident(IdentID),
    Res(TypeID),
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
            MemberID::Type(def) => &p.type_name(def.ty),
        };
        format!("{} '{}'", self.kind(), name)
    }
}

pub enum URes<T> {
    Res(T),
    Unres(IdentID),
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
