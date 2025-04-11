use crate::{
    common::FileSpan,
    ir::{Len, Named, ID},
};

use super::{Type, UInstrInst, UProgram};
use std::{collections::HashMap, fmt::Debug};

pub const NAMED_KINDS: usize = 4;

pub struct UFunc {
    pub args: Vec<VarID>,
    pub ret: Type,
    pub origin: Origin,
    pub instructions: Vec<UInstrInst>,
}

#[derive(Clone)]
pub struct StructField {
    pub name: String,
    pub ty: Type,
}

#[derive(Clone)]
pub struct UStruct {
    pub fields: Vec<StructField>,
    pub field_map: HashMap<String, FieldID>,
    pub origin: Origin,
}

#[derive(Clone)]
pub struct UVar {
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
pub struct UData {
    pub ty: Type,
    pub origin: Origin,
    pub content: Vec<u8>,
}

pub type Origin = FileSpan;

impl UFunc {
    pub fn ty(&self, program: &UProgram) -> Type {
        Type::Fn {
            args: self
                .args
                .iter()
                .map(|a| program.expect(*a).ty.clone())
                .collect(),
            ret: Box::new(self.ret.clone()),
        }
    }
}

impl UStruct {
    pub fn field(&self, id: FieldID) -> &StructField {
        &self.fields[id.0]
    }
    pub fn get_field(&self, name: &str) -> Option<&StructField> {
        self.field_map.get(name).map(|id| self.field(*id))
    }
    pub fn iter_fields(&self) -> impl Iterator<Item = (FieldID, &StructField)> {
        self.fields
            .iter()
            .enumerate()
            .map(|(i, f)| (FieldID::new(i), f))
    }
}

pub type StructID = ID<UStruct>;
pub type VarID = ID<UVar>;
pub type DataID = ID<UData>;
pub type FieldID = ID<StructField>;
pub type FnID = ID<UFunc>;

impl Kind for UFunc {
    const INDEX: usize = 0;
    fn from_program_mut(program: &mut UProgram) -> &mut Vec<Option<Self>> {
        &mut program.fns
    }
    fn from_program(program: &UProgram) -> &Vec<Option<Self>> {
        &program.fns
    }
}
impl Named for UFunc {
    const NAME: &str = "func";
}

impl Kind for UVar {
    const INDEX: usize = 1;
    fn from_program_mut(program: &mut UProgram) -> &mut Vec<Option<Self>> {
        &mut program.vars
    }
    fn from_program(program: &UProgram) -> &Vec<Option<Self>> {
        &program.vars
    }
}
impl Named for UVar {
    const NAME: &str = "var";
}

impl Kind for UStruct {
    const INDEX: usize = 2;
    fn from_program_mut(program: &mut UProgram) -> &mut Vec<Option<Self>> {
        &mut program.structs
    }
    fn from_program(program: &UProgram) -> &Vec<Option<Self>> {
        &program.structs
    }
}
impl Named for UStruct {
    const NAME: &str = "struct";
}

impl Kind for UData {
    const INDEX: usize = 3;
    fn from_program_mut(program: &mut UProgram) -> &mut Vec<Option<Self>> {
        &mut program.data
    }
    fn from_program(program: &UProgram) -> &Vec<Option<Self>> {
        &program.data
    }
}
impl Named for UData {
    const NAME: &str = "data";
}

impl Named for StructField {
    const NAME: &str = "field";
}

pub trait Kind {
    const INDEX: usize;
    fn from_program_mut(program: &mut UProgram) -> &mut Vec<Option<Self>>
    where
        Self: Sized;
    fn from_program(program: &UProgram) -> &Vec<Option<Self>>
    where
        Self: Sized;
}
