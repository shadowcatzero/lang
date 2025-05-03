use super::{GenericID, Len, ModPath, TypeID, UFunc, UProgram, UStruct, UVar, VarID, VarInst};
use crate::ir::ID;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct FieldRef {
    pub parent: VarID,
    pub name: String,
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct GenericTy<T> {
    pub id: ID<T>,
    pub args: Vec<TypeID>,
}

#[derive(Clone)]
pub enum Type {
    Bits(u32),
    Struct(GenericTy<UStruct>),
    Fn(GenericTy<UFunc>),
    Ref(TypeID),
    Deref(TypeID),
    Slice(TypeID),
    Array(TypeID, Len),
    Unit,
    // "fake" types
    Unres(ModPath),
    Generic(GenericID),
    Infer,
    Error,
}

impl Type {
    pub fn rf(self, p: &mut UProgram) -> Self {
        p.def_ty(self).rf()
    }
    pub fn derf(self, p: &mut UProgram) -> Self {
        p.def_ty(self).derf()
    }
    pub fn arr(self, p: &mut UProgram, len: Len) -> Self {
        p.def_ty(self).arr(len)
    }
    pub fn slice(self, p: &mut UProgram) -> Self {
        p.def_ty(self).slice()
    }
}

impl TypeID {
    pub fn rf(self) -> Type {
        Type::Ref(self)
    }
    pub fn derf(self) -> Type {
        Type::Deref(self)
    }
    pub fn arr(self, len: Len) -> Type {
        Type::Array(self, len)
    }
    pub fn slice(self) -> Type {
        Type::Slice(self)
    }
}

impl Type {
    pub fn bx(self) -> Box<Self> {
        Box::new(self)
    }
}

pub trait TypeIDed {
    fn type_id(&self, vars: &[UVar]) -> TypeID;
}

impl TypeIDed for TypeID {
    fn type_id(&self, _: &[UVar]) -> TypeID {
        *self
    }
}

impl TypeIDed for &TypeID {
    fn type_id(&self, _: &[UVar]) -> TypeID {
        **self
    }
}

impl TypeIDed for VarID {
    fn type_id(&self, vars: &[UVar]) -> TypeID {
        vars[self].ty
    }
}

impl TypeIDed for VarInst {
    fn type_id(&self, vars: &[UVar]) -> TypeID {
        self.id.type_id(vars)
    }
}

impl TypeIDed for &VarInst {
    fn type_id(&self, vars: &[UVar]) -> TypeID {
        self.id.type_id(vars)
    }
}
