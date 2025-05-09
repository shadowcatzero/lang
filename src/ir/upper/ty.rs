use super::{FnID, GenericID, Len, ResolveRes, StructID, TypeID, UProgram, VarID};

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
    Bits(u32),
    Struct(StructInst),
    // this can be added for constraints later (F: fn(...) -> ...)
    // Fn { args: Vec<TypeID>, ret: TypeID },
    // "fake" types
    FnInst(FnInst),
    Ref(TypeID),
    Slice(TypeID),
    Array(TypeID, Len),
    Unit,
    Infer,
    Generic(GenericID),
    Deref(TypeID),
    Ptr(TypeID),
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

pub fn clean_type(types: &[Type], id: TypeID) -> Option<TypeID> {
    match &types[id] {
        &Type::Ptr(id) => clean_type(types, id),
        &Type::Deref(did) => match &types[clean_type(types, did)?] {
            &Type::Ref(id) => clean_type(types, id),
            _ => Some(id),
        },
        Type::Error => None,
        _ => Some(id),
    }
}

pub fn resolved_type(types: &[Type], id: TypeID) -> Result<TypeID, ResolveRes> {
    match &types[id] {
        &Type::Ptr(id) => resolved_type(types, id),
        &Type::Deref(id) => match &types[resolved_type(types, id)?] {
            &Type::Ref(id) => resolved_type(types, id),
            Type::Infer => Err(ResolveRes::Unfinished),
            _ => Err(ResolveRes::Finished),
        },
        Type::Error => Err(ResolveRes::Finished),
        _ => Ok(id),
    }
}
