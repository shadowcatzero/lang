use crate::{
    common::FileSpan,
    ir::{FieldID, Len, StructID, VarID},
};

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
    pub name: String,
    pub ty: Type,
}

#[derive(Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<StructField>,
    pub field_map: HashMap<String, FieldID>,
    pub origin: Origin,
}

#[derive(Clone)]
pub struct VarDef {
    pub name: String,
    pub parent: Option<FieldRef>,
    pub ty: Type,
    pub origin: Origin,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct VarOffset {
    pub id: VarID,
    pub offset: Len,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct FieldRef {
    pub var: VarID,
    // this is technically redundant bc you can get it from the var...
    // but it makes things a lot easier, and you'd have to recheck the fields anyways
    pub struc: StructID,
    pub field: FieldID,
}

#[derive(Clone)]
pub struct DataDef {
    pub ty: Type,
    pub origin: Origin,
    pub label: String,
}

pub type Origin = FileSpan;

impl FnDef {
    pub fn ty(&self) -> Type {
        Type::Fn {
            args: self.args.iter().map(|a| a.ty.clone()).collect(),
            ret: Box::new(self.ret.clone()),
        }
    }
}

impl StructDef {
    pub fn field(&self, id: FieldID) -> &StructField {
        &self.fields[id.0]
    }
    pub fn get_field(&self, name: &str) -> Option<&StructField> {
        self.field_map.get(name).map(|id| self.field(*id))
    }
    pub fn iter_fields(&self) -> impl Iterator<Item = (FieldID, &StructField)> {
        self.fields.iter().enumerate().map(|(i, f)| (FieldID(i), f))
    }
}
