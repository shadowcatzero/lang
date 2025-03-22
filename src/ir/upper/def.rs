use crate::common::FileSpan;

use super::Type;
use std::fmt::Debug;

#[derive(Clone)]
pub struct FnDef {
    pub name: String,
    pub args: Vec<VarDef>,
    pub ret: Type,
    pub origin: Origin,
}

#[derive(Clone)]
pub struct TypeDef {
    pub name: String,
    pub args: usize,
    pub origin: Origin,
}

#[derive(Clone)]
pub struct VarDef {
    pub name: String,
    pub ty: Type,
    pub origin: Origin,
}

#[derive(Clone)]
pub struct DataDef {
    pub ty: Type,
    pub origin: Origin,
}

#[derive(Debug, Clone, Copy)]
pub enum Origin {
    Builtin,
    File(FileSpan),
}

impl FnDef {
    pub fn ty(&self) -> Type {
        Type::Fn {
            args: self.args.iter().map(|a| a.ty.clone()).collect(),
            ret: Box::new(self.ret.clone()),
        }
    }
}
