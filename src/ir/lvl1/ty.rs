use super::{Origin, TypeDef, TypeIdent};

#[derive(Clone)]
pub enum Type {
    Concrete(TypeIdent),
    Generic { base: TypeIdent, args: Vec<Type> },
    Fn { args: Vec<Type>, ret: Box<Type> },
    Ref(Box<Type>),
    Infer,
    Error,
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
    pub fn id(&self) -> TypeIdent {
        TypeIdent::builtin(self)
    }
}
