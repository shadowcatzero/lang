use super::*;
use std::collections::HashMap;

pub struct UProgram {
    // kinds
    pub fns: Vec<Option<UFunc>>,
    pub vars: Vec<Option<UVar>>,
    pub structs: Vec<Option<UStruct>>,
    pub types: Vec<Option<Type>>,
    pub data: Vec<Option<UData>>,
    // associated data
    pub names: NameMap,
    pub origins: OriginMap,
    pub fn_var: FnVarMap,
    // utils for creation
    error: Option<TypeID>,
    pub path: Vec<String>,
    pub name_stack: Vec<HashMap<String, Idents>>,
    pub temp: usize,
}

impl UProgram {
    pub fn new() -> Self {
        Self {
            fns: Vec::new(),
            vars: Vec::new(),
            structs: Vec::new(),
            types: Vec::new(),
            data: Vec::new(),
            names: NameMap::new(),
            origins: OriginMap::new(),
            fn_var: FnVarMap::new(),
            error: None,
            path: Vec::new(),
            name_stack: Vec::new(),
            temp: 0,
        }
    }
    pub fn error_type(&mut self) -> TypeID {
        self.error
            .unwrap_or_else(|| self.def("error", Some(Type::Error), Origin::builtin()))
    }
    pub fn set_module(&mut self, path: Vec<String>) {
        self.path = path;
        self.name_stack = vec![HashMap::new()];
    }
    pub fn push(&mut self) {
        self.name_stack.push(HashMap::new());
    }
    pub fn pop(&mut self) {
        self.name_stack.pop();
    }
    pub fn push_name(&mut self, name: &str) {
        self.path.push(name.to_string());
    }
    pub fn pop_name(&mut self) {
        self.path.pop();
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
    pub fn get<K: Kind>(&self, id: ID<K>) -> Option<&K> {
        K::from_program(self)[id.0].as_ref()
    }
    pub fn get_mut<K: Kind>(&mut self, id: ID<K>) -> Option<&mut K> {
        K::from_program_mut(self)[id.0].as_mut()
    }
    pub fn expect<K: Kind + Named>(&self, id: ID<K>) -> &K {
        self.get(id)
            .unwrap_or_else(|| panic!("{id:?} not defined yet!"))
    }
    pub fn expect_type(&self, var: VarID) -> &Type {
        self.expect(self.expect(var).ty)
    }
    pub fn expect_mut<K: Kind + Named>(&mut self, id: ID<K>) -> &mut K {
        self.get_mut(id)
            .unwrap_or_else(|| panic!("{id:?} not defined yet!"))
    }
    pub fn get_fn_var(&self, id: VarID) -> Option<&UFunc> {
        self.fns[self.fn_var.fun(id)?.0].as_ref()
    }

    pub fn temp_var(&mut self, origin: Origin, ty: TypeID) -> VarInst {
        self.temp_var_inner(origin, ty)
    }

    fn temp_var_inner(&mut self, origin: Origin, ty: TypeID) -> VarInst {
        let v = self.def(&format!("temp{}", self.temp), Some(UVar { ty }), origin);
        self.temp += 1;
        VarInst {
            id: v,
            span: origin,
        }
    }

    pub fn write<K: Kind>(&mut self, id: ID<K>, k: K) {
        K::from_program_mut(self)[id.0] = Some(k);
    }

    pub fn def<K: Kind + Finish>(&mut self, name: &str, k: Option<K>, origin: Origin) -> ID<K> {
        self.names.push::<K>(&self.path_for(name));
        self.origins.push::<K>(origin);
        let vec = K::from_program_mut(self);
        let id = ID::new(vec.len());
        vec.push(k);
        K::finish(self, id, name);
        id
    }

    pub fn path_for(&self, name: &str) -> Vec<String> {
        if self.path.is_empty() {
            return vec![name.to_string()];
        }
        let mut path = self.path.clone();
        path.push(name.to_string());
        path
    }

    pub fn def_searchable<K: Kind + Finish>(
        &mut self,
        name: &str,
        k: Option<K>,
        origin: Origin,
    ) -> ID<K> {
        let id = self.def(&name, k, origin);
        self.name_on_stack(id, name.to_string());
        id
    }

    // hopefully these are not needed after redoing types
    // pub fn field_type<'a>(&'a self, ty: &'a Type, field: &str) -> Option<&'a Type> {
    //     if let Type::Struct(sty) = ty {
    //         Some(&self.get(*sty.fields.get(field)?)?.ty)
    //     } else if let Type::Module(path) = ty {
    //         let id = self.names.id::<UVar>(path, field)?;
    //         Some(&self.get(id)?.ty)
    //     } else {
    //         None
    //     }
    // }
    //
    // pub fn follow_ref(&self, m: &FieldRef) -> Option<&Type> {
    //     let parent = self.get(m.parent)?;
    //     self.field_type(self.follow_type(&parent.ty)?, &m.name)
    // }
    //
    // pub fn get_type<'a>(&'a self, v: VarID) -> Option<&'a Type> {
    //     self.follow_type(&self.get(v)?.ty)
    // }
    //
    // pub fn set_type(&mut self, mut var: VarID, set: Type) -> Option<()> {
    //     let mut path = Vec::new();
    //     while let Type::Field(parent) = &self.get(var)?.ty {
    //         var = parent.parent;
    //         path.push(parent.name.clone());
    //     }
    //     let mut ty = &mut self.vars[var.0].as_mut()?.ty;
    //     'outer: while let Type::Struct(sty) = ty {
    //         let Some(name) = path.pop() else {
    //             break;
    //         };
    //         let struc = &self.structs[sty.id.0].as_ref()?;
    //         let field = struc.fields.get(&name)?;
    //         let Type::Generic { id } = field.ty else {
    //             return None;
    //         };
    //         for (i, g) in struc.generics.iter().enumerate() {
    //             if *g == id {
    //                 ty = &mut sty.args[i];
    //                 continue 'outer;
    //             }
    //         }
    //         return None;
    //     }
    //     *ty = set;
    //     Some(())
    // }
    //
    // pub fn follow_type<'a>(&'a self, ty: &'a Type) -> Option<&'a Type> {
    //     match ty {
    //         Type::Field(m) => {
    //             let parent = self.get(m.parent)?;
    //             self.field_type(self.follow_type(&parent.ty)?, &m.name)
    //         }
    //         ty => Some(ty),
    //     }
    // }

    pub fn type_name(&self, ty: &Type) -> String {
        let mut str = String::new();
        match ty {
            Type::Struct(ty) => {
                let base = ty.id;
                let args = &ty.args;
                str += self.names.name(base);
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
            Type::Generic { id } => str += self.names.name(*id),
            Type::Bits(size) => str += &format!("b{}", size),
            Type::Array(t, len) => str += &format!("[{}; {len}]", self.type_name(t)),
            Type::Unit => str += "()",
            Type::Slice(t) => str += &format!("&[{}]", self.type_name(t)),
            Type::Error => str += "{error}",
            Type::Infer => str += "{inferred}",
            Type::Placeholder => str += "{placeholder}",
            Type::Module(path) => str += &path.join("::"),
            Type::Field(m) => {
                str += &self
                    .follow_ref(m)
                    .map(|t| self.type_name(t))
                    .unwrap_or("{error}".to_string())
            }
        }
        str
    }
    pub fn path_var(&self, path: &NamePath, name: &str) -> Option<VarID> {
        self.names.id(path, name)
    }
    pub fn path_ty(&self, path: &NamePath, name: &str) -> Option<&Type> {
        Some(&self.get(self.path_var(path, name)?)?.ty)
    }
    fn name_on_stack<K: Kind>(&mut self, id: ID<K>, name: String) {
        let idx = self.name_stack.len() - 1;
        let last = &mut self.name_stack[idx];
        if let Some(l) = last.get_mut(&name) {
            l.insert(id.into());
        } else {
            last.insert(name, Idents::new(id.into()));
        }
    }
    pub fn iter_vars(&self) -> impl Iterator<Item = (VarID, &UVar)> {
        self.vars
            .iter()
            .flatten()
            .enumerate()
            .map(|(i, x)| (ID::new(i), x))
    }
    pub fn iter_fns(&self) -> impl Iterator<Item = (FnID, &UFunc)> {
        self.fns
            .iter()
            .flatten()
            .enumerate()
            .map(|(i, x)| (ID::new(i), x))
    }
    pub fn cloned_fns(&self) -> impl Iterator<Item = (FnID, UFunc)> + use<'_> {
        self.fns
            .iter()
            .flatten()
            .enumerate()
            .map(|(i, x)| (ID::new(i), x.clone()))
    }
}
