use super::{FnID, GenericID, IdentID, Len, ResolveRes, StructID, TypeID, UProgram, VarID};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct FieldRef {
    pub parent: VarID,
    pub name: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StructInst {
    pub id: StructID,
    /// assumed to be valid
    pub gargs: Vec<TypeID>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FnInst {
    pub id: FnID,
    /// assumed to be valid
    pub gargs: Vec<TypeID>,
}

#[derive(Clone, PartialEq)]
pub enum Type {
    Real(RType),
    Deref(TypeID),
    Ptr(TypeID),
    Unres(IdentID),
    Error,
}

/// "real" types
#[derive(Clone, PartialEq)]
pub enum RType {
    Bits(u32),
    Struct(StructInst),
    // this can be added for constraints later (F: fn(...) -> ...)
    // Fn { args: Vec<TypeID>, ret: TypeID },
    // "fake" types
    FnRef(FnInst),
    Ref(TypeID),
    Slice(TypeID),
    Array(TypeID, Len),
    Unit,
    Infer,
    Generic(GenericID),
}

impl RType {
    pub const fn ty(self) -> Type {
        Type::Real(self)
    }
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
        RType::Ref(self).ty()
    }
    pub fn derf(self) -> Type {
        Type::Deref(self)
    }
    pub fn arr(self, len: Len) -> Type {
        RType::Array(self, len).ty()
    }
    pub fn slice(self) -> Type {
        RType::Slice(self).ty()
    }
}

impl Type {
    pub fn bx(self) -> Box<Self> {
        Box::new(self)
    }
}

pub fn real_type(types: &[Type], id: TypeID) -> Result<&RType, ResolveRes> {
    match &types[id] {
        Type::Real(rtype) => Ok(rtype),
        &Type::Ptr(id) => real_type(types, id),
        &Type::Deref(id) => match real_type(types, id)? {
            &RType::Ref(id) => real_type(types, id),
            _ => Err(ResolveRes::Finished),
        },
        Type::Unres(_) => Err(ResolveRes::Unfinished),
        Type::Error => Err(ResolveRes::Finished),
    }
}
