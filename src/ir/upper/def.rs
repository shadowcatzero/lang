use crate::{common::FileSpan, ir::{Len, Size}};

use super::Type;
use std::{collections::HashMap, fmt::Debug};

#[derive(Clone)]
pub struct FnDef {
    pub name: String,
    pub args: Vec<VarDef>,
    pub ret: Type,
    pub origin: Origin,
}

#[derive(Clone)]
pub struct StructField {
    pub ty: Type,
    pub offset: Len,
}

#[derive(Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: HashMap<String, StructField>,
    pub size: Size,
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
    pub label: String,
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
