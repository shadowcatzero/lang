use super::*;
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

pub struct UProgram {
    pub fns: Vec<UFunc>,
    pub structs: Vec<UStruct>,
    pub modules: Vec<Option<UModule>>,
    pub data: Vec<UData>,
    pub generics: Vec<UGeneric>,
    pub vars: Vec<UVar>,
    pub types: Vec<Type>,
}

pub struct UModuleBuilder<'a> {
    pub p: &'a mut UProgram,
    pub module: ModID,
    pub temp: usize,
}

impl UProgram {
    pub fn new() -> Self {
        Self {
            fns: Vec::new(),
            vars: Vec::new(),
            structs: Vec::new(),
            types: Vec::new(),
            generics: Vec::new(),
            data: Vec::new(),
            modules: Vec::new(),
        }
    }

    pub fn instantiate_type(&mut self, ty: Type) {
        self.def_ty(match ty {
            Type::Ref(node) => Type::Ref(node.lower()),
            Type::Generic(node, nodes) => todo!(),
            Type::Ident(node) => todo!(),
        });
    }

    pub fn infer(&mut self) -> TypeID {
        self.def_ty(Type::Infer)
    }

    pub fn error(&mut self) -> TypeID {
        self.def_ty(Type::Error)
    }

    pub fn def_var(&mut self, v: UVar) -> VarID {
        Self::push_id(&mut self.vars, v)
    }

    pub fn def_fn(&mut self, f: UFunc) -> FnID {
        Self::push_id(&mut self.fns, f)
    }

    pub fn def_ty(&mut self, t: Type) -> TypeID {
        Self::push_id(&mut self.types, t)
    }

    pub fn def_generic(&mut self, g: UGeneric) -> GenericID {
        Self::push_id(&mut self.generics, g)
    }

    pub fn def_data(&mut self, d: UData) -> DataID {
        Self::push_id(&mut self.data, d)
    }

    pub fn def_struct(&mut self, s: UStruct) -> StructID {
        Self::push_id(&mut self.structs, s)
    }

    fn push_id<T>(v: &mut Vec<T>, t: T) -> ID<T> {
        let id = ID::new(v.len());
        v.push(t);
        id
    }

    pub fn type_name<T: Typer>(&self, ty: T) -> String {
        let mut str = String::new();
        match ty.ty(self) {
            Type::Struct(ty) => {
                str += &self.structs[ty.id].name;
                let args = &ty.args;
                if let Some(arg) = args.first() {
                    str = str + "<" + &self.type_name(arg);
                }
                for arg in args.iter().skip(1) {
                    str = str + ", " + &self.type_name(arg);
                }
                if !args.is_empty() {
                    str += ">";
                }
            }
            Type::Fn { args, ret } => {
                str += "fn(";
                if let Some(arg) = args.first() {
                    str += &self.type_name(arg);
                }
                for arg in args.iter().skip(1) {
                    str = str + ", " + &self.type_name(arg);
                }
                str += ") -> ";
                str += &self.type_name(ret);
            }
            Type::Ref(t) => {
                str += &self.type_name(t);
                str += "&";
            }
            Type::Deref(t) => {
                str += &self.type_name(t);
                str += "^";
            }
            Type::Unres(_) => {
                str += "{unresolved}";
            }
            Type::Bits(size) => str += &format!("b{}", size),
            Type::Array(t, len) => str += &format!("[{}; {len}]", self.type_name(t)),
            Type::Unit => str += "()",
            Type::Slice(t) => str += &format!("&[{}]", self.type_name(t)),
            Type::Error => str += "{error}",
            Type::Infer => str += "{inferred}",
            Type::Placeholder => str += "{placeholder}",
        }
        str
    }
}

impl<'a> UModuleBuilder<'a> {
    pub fn new(program: &'a mut UProgram, id: ModID, error: TypeID) -> Self {
        Self {
            p: program,
            module: id,
            error,
            name_stack: Vec::new(),
            temp: 0,
        }
    }
    pub fn push(&mut self) {
        self.name_stack.push(HashMap::new());
    }
    pub fn pop(&mut self) {
        self.name_stack.pop();
    }
    pub fn get_idents(&self, name: &str) -> Option<Idents> {
        for map in self.name_stack.iter().rev() {
            let res = map.get(name);
            if res.is_some() {
                return res.cloned();
            }
        }
        None
    }

    pub fn def_var(&mut self, name: &str, v: UVar, origin: Origin) -> VarID {
        let id = self.p.def_var(name, v, origin);
        self.name_on_stack(id, name.to_string());
        id
    }

    pub fn temp_var<T: Typable>(&mut self, origin: Origin, ty: T) -> VarInst {
        self.temp_var_inner(origin, ty)
    }
    fn temp_var_inner<T: Typable>(&mut self, origin: Origin, ty: T) -> VarInst {
        let t = self.temp;
        let v = self
            .p
            .def_var(&format!("temp{}", t), UVar { ty: ty.ty(self) }, origin);
        self.temp += 1;
        VarInst { id: v, origin }
    }
}

// I'm done with names...
pub trait Typer {
    fn ty<'a>(&'a self, p: &'a UProgram) -> &'a Type;
}

impl Typer for &Type {
    fn ty(&self, _: &UProgram) -> &Type {
        self
    }
}

impl Typer for TypeID {
    fn ty<'a>(&'a self, p: &'a UProgram) -> &'a Type {
        &p.types[self]
    }
}

impl Typer for &TypeID {
    fn ty<'a>(&'a self, p: &'a UProgram) -> &'a Type {
        &p.types[*self]
    }
}

impl Typer for &Box<Type> {
    fn ty<'a>(&'a self, _: &'a UProgram) -> &'a Type {
        &**self
    }
}

pub trait Typable {
    fn ty(self, p: &mut UProgram) -> TypeID;
}

impl Typable for Type {
    fn ty(self, p: &mut UProgram) -> TypeID {
        p.def_ty(self)
    }
}

impl Typable for TypeID {
    fn ty(self, p: &mut UProgram) -> TypeID {
        self
    }
}

impl Deref for UModuleBuilder<'_> {
    type Target = UProgram;

    fn deref(&self) -> &Self::Target {
        self.p
    }
}

impl DerefMut for UModuleBuilder<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.p
    }
}
