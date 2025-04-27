use super::{Type, UInstrInst, UInstruction, UProgram};
use crate::{
    common::FileSpan,
    ir::{Len, Named, ID},
};
use std::{collections::HashMap, fmt::Debug};

#[derive(Clone)]
pub struct UFunc {
    pub args: Vec<VarID>,
    pub ret: Type,
    pub instructions: Vec<UInstrInst>,
}

#[derive(Clone)]
pub struct StructField {
    pub ty: Type,
}

#[derive(Clone)]
pub struct UStruct {
    pub fields: HashMap<String, StructField>,
    pub generics: Vec<GenericID>,
}

#[derive(Clone)]
pub struct UGeneric {}

#[derive(Clone)]
pub struct UVar {
    pub ty: Type,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct VarOffset {
    pub id: VarID,
    pub offset: Len,
}

#[derive(Clone)]
pub struct UData {
    pub ty: Type,
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

macro_rules! impl_kind {
    // TRUST THIS IS SANE!!! KEEP THE CODE DRY AND SAFE!!!!!!
    ($struc:ty, $idx:expr, $field:ident, $name:expr) => {
        impl_kind!($struc, $idx, $field, $name, nofin);
        impl Finish for $struc {
            fn finish(_: &mut UProgram, _: ID<Self>, _: &str) {}
        }
    };
    ($struc:ty, $idx:expr, $field:ident, $name:expr, nofin) => {
        impl Kind for $struc {
            const INDEX: usize = $idx;
            fn from_program_mut(program: &mut UProgram) -> &mut Vec<Option<Self>> {
                &mut program.$field
            }
            fn from_program(program: &UProgram) -> &Vec<Option<Self>> {
                &program.$field
            }
        }
        impl Named for $struc {
            const NAME: &str = $name;
        }
    };
}

impl_kind!(UFunc, 0, fns, "func", nofin);
impl_kind!(UVar, 1, vars, "var");
impl_kind!(UStruct, 2, structs, "struct");
impl_kind!(UGeneric, 3, types, "type");
impl_kind!(UData, 4, data, "data");
pub const NAMED_KINDS: usize = 6;

pub type FnID = ID<UFunc>;
pub type VarID = ID<UVar>;
pub type StructID = ID<UStruct>;
pub type DataID = ID<UData>;
pub type GenericID = ID<UGeneric>;

impl Finish for UFunc {
    fn finish(p: &mut UProgram, id: ID<Self>, name: &str) {
        let var = p.def_searchable(
            name,
            Some(UVar {
                ty: Type::Placeholder,
            }),
            p.origins.get(id),
        );
        p.fn_var.insert(id, var);
    }
}

pub trait Kind: Sized {
    const INDEX: usize;
    fn from_program_mut(program: &mut UProgram) -> &mut Vec<Option<Self>>;
    fn from_program(program: &UProgram) -> &Vec<Option<Self>>;
}

pub trait Finish: Sized {
    fn finish(program: &mut UProgram, id: ID<Self>, name: &str);
}
