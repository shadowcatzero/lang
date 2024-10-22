use std::{
    collections::HashMap,
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use super::{BuiltinType, FileSpan, FnDef, Function, Type, TypeDef, VarDef};

pub struct Namespace {
    pub fn_defs: Vec<FnDef>,
    pub var_defs: Vec<VarDef>,
    pub type_defs: Vec<TypeDef>,
    pub fns: Vec<Option<Function>>,
    pub temp: usize,
    pub stack: Vec<HashMap<String, Idents>>,
}

impl Namespace {
    pub fn new() -> Self {
        let mut s = Self {
            fn_defs: Vec::new(),
            var_defs: Vec::new(),
            type_defs: Vec::new(),
            fns: Vec::new(),
            temp: 0,
            stack: vec![HashMap::new()],
        };
        for b in BuiltinType::enumerate() {
            s.def_type(b.def());
        }
        s
    }
    pub fn push(&mut self) -> NamespaceGuard {
        self.stack.push(HashMap::new());
        NamespaceGuard(self)
    }
    pub fn get(&self, name: &str) -> Option<Idents> {
        for map in self.stack.iter().rev() {
            let res = map.get(name);
            if res.is_some() {
                return res.cloned();
            }
        }
        None
    }
    pub fn get_var(&self, id: VarIdent) -> &VarDef {
        &self.var_defs[id.0]
    }
    pub fn get_fn(&self, id: FnIdent) -> &FnDef {
        &self.fn_defs[id.0]
    }
    pub fn get_type(&self, id: TypeIdent) -> &TypeDef {
        &self.type_defs[id.0]
    }
    pub fn alias_fn(&mut self, name: &str, id: FnIdent) {
        self.insert(name, Ident::Fn(id));
    }
    pub fn name_var(&mut self, def: &VarDef, var: VarIdent) {
        self.insert(&def.name, Ident::Var(var));
    }
    pub fn def_var(&mut self, var: VarDef) -> VarIdent {
        let i = self.var_defs.len();
        self.var_defs.push(var);
        VarIdent(i)
    }
    pub fn temp_var(&mut self, origin: FileSpan, ty: Type) -> VarIdent {
        let v = self.def_var(VarDef {
            name: format!("temp{}", self.temp),
            origin: super::Origin::File(origin),
            ty,
        });
        self.temp += 1;
        v
    }
    pub fn def_fn(&mut self, def: FnDef) -> FnIdent {
        let i = self.fn_defs.len();
        let id = FnIdent(i);
        self.insert(&def.name, Ident::Fn(id));
        self.fn_defs.push(def);
        self.fns.push(None);
        id
    }
    pub fn def_type(&mut self, def: TypeDef) -> TypeIdent {
        let i = self.type_defs.len();
        let id = TypeIdent(i);
        self.insert(&def.name, Ident::Type(id));
        self.type_defs.push(def);
        id
    }
    pub fn type_name(&self, ty: &Type) -> String {
        let mut str = String::new();
        match ty {
            Type::Concrete(t) => {
                str += &self.get_type(*t).name;
            }
            Type::Generic { base, args } => {
                str += &self.get_type(*base).name;
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
                str = str + "&" + &self.type_name(t);
            }
            Type::Error => str += "{error}",
            Type::Infer => str += "{inferred}",
        }
        str
    }
    fn insert(&mut self, name: &str, id: Ident) {
        let idx = self.stack.len() - 1;
        let last = &mut self.stack[idx];
        if let Some(l) = last.get_mut(name) {
            l.insert(id);
        } else {
            last.insert(name.to_string(), Idents::new(id));
        }
    }
    pub fn write_fn(&mut self, id: FnIdent, f: Function) {
        self.fns[id.0] = Some(f);
    }
}

pub struct NamespaceGuard<'a>(&'a mut Namespace);

impl Drop for NamespaceGuard<'_> {
    fn drop(&mut self) {
        self.0.stack.pop();
    }
}

impl Deref for NamespaceGuard<'_> {
    type Target = Namespace;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl DerefMut for NamespaceGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Ident {
    Var(VarIdent),
    Fn(FnIdent),
    Type(TypeIdent),
}

#[derive(Debug, Clone, Copy)]
pub struct Idents {
    pub latest: Ident,
    pub var: Option<VarIdent>,
    pub func: Option<FnIdent>,
    pub var_func: Option<VarOrFnIdent>,
    pub ty: Option<TypeIdent>,
}

impl Idents {
    fn new(latest: Ident) -> Self {
        let mut s = Self {
            latest,
            var: None,
            func: None,
            var_func: None,
            ty: None,
        };
        s.insert(latest);
        s
    }
    fn insert(&mut self, i: Ident) {
        self.latest = i;
        match i {
            Ident::Var(v) => {
                self.var = Some(v);
                self.var_func = Some(VarOrFnIdent::Var(v));
            }
            Ident::Fn(f) => {
                self.func = Some(f);
                self.var_func = Some(VarOrFnIdent::Fn(f));
            }
            Ident::Type(t) => self.ty = Some(t),
        }
    }
}

#[derive(Clone, Copy)]
pub struct TypeIdent(usize);
#[derive(Clone, Copy)]
pub struct VarIdent(usize);
#[derive(Clone, Copy)]
pub struct FnIdent(usize);

#[derive(Debug, Clone, Copy)]
pub enum VarOrFnIdent {
    Var(VarIdent),
    Fn(FnIdent),
}

impl TypeIdent {
    pub fn builtin(ty: &BuiltinType) -> Self {
        Self(*ty as usize)
    }
}

impl Debug for VarIdent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "v{}", self.0)
    }
}

impl Debug for TypeIdent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "t{}", self.0)
    }
}

impl Debug for FnIdent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "f{}", self.0)
    }
}
