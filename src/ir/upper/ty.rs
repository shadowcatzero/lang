use super::{IRUProgram, Len, TypeID};

#[derive(Clone, PartialEq)]
pub enum Type {
    Concrete(TypeID),
    Bits(u32),
    Generic { base: TypeID, args: Vec<Type> },
    Fn { args: Vec<Type>, ret: Box<Type> },
    Ref(Box<Type>),
    Slice(Box<Type>),
    Array(Box<Type>, Len),
    Infer,
    Error,
    Unit,
}

impl Type {
    pub fn rf(self) -> Self {
        Self::Ref(Box::new(self))
    }
    pub fn arr(self, len: Len) -> Self {
        Self::Array(Box::new(self), len)
    }
    pub fn slice(self) -> Self {
        Self::Slice(Box::new(self))
    }
}

pub fn resolve_types(ns: &IRUProgram) {
    for (i, f) in ns.iter_fns() {
        for inst in &f.instructions {
            match &inst.i {
                _ => todo!(),
            }
        }
    }
}
