use super::{Origin, TypeDef, TypeID};

#[derive(Clone)]
pub enum Type {
    Concrete(TypeID),
    Bits(u32),
    Generic { base: TypeID, args: Vec<Type> },
    Fn { args: Vec<Type>, ret: Box<Type> },
    Ref(Box<Type>),
    Array(Box<Type>),
    Infer,
    Error,
}

impl Type {
    pub fn rf(self) -> Self {
        Self::Ref(Box::new(self))
    }
    pub fn arr(self) -> Self {
        Self::Array(Box::new(self))
    }
}

#[repr(usize)]
#[derive(Debug, Clone, Copy)]
pub enum BuiltinType {
    Unit,
}

impl BuiltinType {
    pub fn enumerate() -> &'static [Self; 1] {
        &[Self::Unit]
    }
    pub fn def(&self) -> TypeDef {
        match self {
            BuiltinType::Unit => TypeDef {
                name: "()".to_string(),
                args: 0,
                origin: Origin::Builtin,
            },
        }
    }
    pub fn id(&self) -> TypeID {
        TypeID::builtin(self)
    }
}

impl TypeID {
    pub fn builtin(ty: &BuiltinType) -> Self {
        Self(*ty as usize)
    }
}

