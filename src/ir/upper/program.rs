use super::*;

pub struct UProgram {
    pub fns: Vec<UFunc>,
    pub structs: Vec<UStruct>,
    pub modules: Vec<UModule>,
    pub data: Vec<UData>,
    pub generics: Vec<UGeneric>,
    pub vars: Vec<UVar>,
    pub idents: Vec<UIdent>,
    pub types: Vec<Type>,
    pub instrs: Vec<UInstrInst>,

    pub unres_idents: Vec<IdentID>,
    pub unres_instrs: Vec<(FnID, InstrID)>,
    pub tc: TypeCache,
}

pub struct TypeCache {
    pub unit: TypeID,
    pub error: TypeID,
}

impl UProgram {
    pub fn new() -> Self {
        let mut types = Vec::new();
        let tc = TypeCache {
            unit: push_id(&mut types, Type::Unit),
            error: push_id(&mut types, Type::Error),
        };
        Self {
            fns: Vec::new(),
            vars: Vec::new(),
            idents: Vec::new(),
            structs: Vec::new(),
            types: Vec::new(),
            generics: Vec::new(),
            data: Vec::new(),
            modules: Vec::new(),
            instrs: Vec::new(),
            unres_idents: Vec::new(),
            unres_instrs: Vec::new(),
            tc,
        }
    }

    pub fn infer(&mut self) -> TypeID {
        self.def_ty(Type::Infer)
    }

    pub fn def_var(&mut self, v: UVar) -> VarID {
        push_id(&mut self.vars, v)
    }

    pub fn def_fn(&mut self, f: UFunc) -> FnID {
        push_id(&mut self.fns, f)
    }

    pub fn def_ty(&mut self, t: Type) -> TypeID {
        push_id(&mut self.types, t)
    }

    pub fn def_ident(&mut self, i: UIdent) -> IdentID {
        let id = push_id(&mut self.idents, i);
        if let IdentStatus::Unres { .. } = self.idents[id].status {
            self.unres_idents.push(id);
        }
        id
    }

    pub fn def_generic(&mut self, g: UGeneric) -> GenericID {
        push_id(&mut self.generics, g)
    }

    pub fn def_data(&mut self, d: UData) -> DataID {
        push_id(&mut self.data, d)
    }

    pub fn def_struct(&mut self, s: UStruct) -> StructID {
        push_id(&mut self.structs, s)
    }

    pub fn def_module(&mut self, m: UModule) -> ModID {
        push_id(&mut self.modules, m)
    }

    pub fn res_ty(&self, i: IdentID) -> Option<TypeID> {
        self.idents[i].status;
    }

    pub fn type_name(&self, ty: impl Typed) -> String {
        match ty.ty(self) {
            Type::Struct(ty) => {
                format!(
                    "{}{}",
                    self.structs[ty.id].name,
                    self.gparams_str(&ty.gargs)
                )
            }
            Type::FnInst(ty) => {
                format!(
                    "fn{}({}) -> {}",
                    &self.gparams_str(&ty.gargs),
                    &self.type_list_str(self.fns[ty.id].args.iter().map(|v| self.vars[v].ty)),
                    &self.type_name(self.fns[ty.id].ret)
                )
            }
            Type::Ref(t) => format!("{}&", self.type_name(t)),
            Type::Bits(size) => format!("b{}", size),
            Type::Array(t, len) => format!("[{}; {len}]", self.type_name(t)),
            Type::Unit => "()".to_string(),
            Type::Slice(t) => format!("&[{}]", self.type_name(t)),
            Type::Infer => "{inferred}".to_string(),
            Type::Generic(id) => self.generics[id].name.clone(),
            Type::Deref(t) => format!("{}^", self.type_name(t)),
            Type::Error => "{error}".to_string(),
            Type::Ptr(id) => self.type_name(id),
        }
    }

    pub fn type_list_str(&self, mut args: impl Iterator<Item = TypeID>) -> String {
        let mut str = String::new();
        if let Some(arg) = args.next() {
            str += &self.type_name(arg);
        }
        for arg in args {
            str = str + ", " + &self.type_name(arg);
        }
        str
    }

    pub fn gparams_str(&self, args: &[TypeID]) -> String {
        let mut str = String::new();
        if !args.is_empty() {
            str += "<";
        }
        str += &self.type_list_str(args.iter().cloned());
        if !args.is_empty() {
            str += ">";
        }
        str
    }
}

pub fn push_id<T>(v: &mut Vec<T>, t: T) -> ID<T> {
    let id = ID::new(v.len());
    v.push(t);
    id
}

// I'm done with names...
pub trait Typed {
    fn ty<'a>(&'a self, p: &'a UProgram) -> &'a Type;
}

impl Typed for &Type {
    fn ty(&self, _: &UProgram) -> &Type {
        self
    }
}

impl Typed for TypeID {
    fn ty<'a>(&'a self, p: &'a UProgram) -> &'a Type {
        &p.types[self]
    }
}

impl Typed for &TypeID {
    fn ty<'a>(&'a self, p: &'a UProgram) -> &'a Type {
        &p.types[*self]
    }
}

impl Typed for &Box<Type> {
    fn ty<'a>(&'a self, _: &'a UProgram) -> &'a Type {
        &**self
    }
}
