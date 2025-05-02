use super::{FnID, GenericID, Len, ModPath, StructID, TypeID, UVar, VarID, VarInst};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct FieldRef {
    pub parent: VarID,
    pub name: String,
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct StructTy {
    pub id: StructID,
    pub args: Vec<TypeID>,
}

#[derive(Clone)]
pub enum Type {
    Bits(u32),
    Struct(StructTy),
    Fn(FnID),
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
    Placeholder,
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
    pub fn is_resolved(&self) -> bool {
        !matches!(self, Self::Error | Self::Placeholder | Self::Infer)
    }
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
