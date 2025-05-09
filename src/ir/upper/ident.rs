use std::fmt::Display;

use super::*;

/// a generic identifier for all (identifiable) kinds
/// eg. a::b::c.d.e
/// or a::Result<T,_>
pub struct UIdent {
    pub status: IdentStatus,
    pub origin: Origin,
}

pub enum IdentStatus {
    Res(Res),
    // lets you do things like import and then specialize in multiple places
    // eg. import SomeStruct ...... f() -> SomeStruct // type ....... SomeStruct {} // struct
    // and then have correct errors like "expected struct, found type Bla"
    Ref(IdentID),
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
