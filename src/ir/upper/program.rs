use super::*;
use std::collections::HashMap;

pub struct UProgram {
    pub fns: Vec<Option<UFunc>>,
    pub vars: Vec<Option<UVar>>,
    pub structs: Vec<Option<UStruct>>,
    pub types: Vec<Option<UGeneric>>,
    pub data: Vec<Option<UData>>,
    pub names: NameMap,
    pub origins: OriginMap,
    pub fn_var: FnVarMap,
    pub temp: usize,
    pub name_stack: Vec<HashMap<String, Idents>>,
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
            temp: 0,
            name_stack: vec![HashMap::new()],
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
    pub fn expect_mut<K: Kind + Named>(&mut self, id: ID<K>) -> &mut K {
        self.get_mut(id)
            .unwrap_or_else(|| panic!("{id:?} not defined yet!"))
    }
    pub fn get_fn_var(&self, id: VarID) -> Option<&UFunc> {
        self.fns[self.fn_var.fun(id)?.0].as_ref()
    }
    pub fn temp_subvar(&mut self, origin: Origin, ty: Type, parent: FieldRef) -> VarInst {
        self.temp_var_inner(origin, ty, Some(parent))
    }
    pub fn temp_var(&mut self, origin: Origin, ty: Type) -> VarInst {
        self.temp_var_inner(origin, ty, None)
    }

    fn temp_var_inner(&mut self, origin: Origin, ty: Type, parent: Option<FieldRef>) -> VarInst {
        let v = self.def(
            format!("temp{}", self.temp),
            Some(UVar { parent, ty }),
            origin,
        );
        self.temp += 1;
        VarInst {
            id: v,
            span: origin,
        }
    }

    pub fn write<K: Kind>(&mut self, id: ID<K>, k: K) {
        K::from_program_mut(self)[id.0] = Some(k);
    }

    pub fn def<K: Kind + Finish>(&mut self, name: String, k: Option<K>, origin: Origin) -> ID<K> {
        self.names.push::<K>(name);
        self.origins.push::<K>(origin);
        let vec = K::from_program_mut(self);
        let id = ID::new(vec.len());
        vec.push(k);
        K::finish(self, id);
        id
    }

    pub fn def_searchable<K: Kind + Finish>(
        &mut self,
        name: String,
        k: Option<K>,
        origin: Origin,
    ) -> ID<K> {
        let id = self.def(name.clone(), k, origin);
        self.name_on_stack(id, name);
        id
    }

    pub fn field_type<'a>(&'a self, sty: &'a Type, field: &str) -> Option<&'a Type> {
        let Type::Struct { id, args } = sty else {
            return None;
        };
        let struc = self.get(*id)?;
        let field = struc.fields.get(field)?;
        if let Type::Generic { id } = field.ty {
            for (i, g) in struc.generics.iter().enumerate() {
                if *g == id {
                    return Some(&args[i]);
                }
            }
        }
        Some(&field.ty)
    }

    pub fn type_name(&self, ty: &Type) -> String {
        let mut str = String::new();
        match ty {
            Type::Struct { id: base, args } => {
                str += self.names.name(*base);
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
            Type::Generic { id } => str += self.names.name(*id),
            Type::Bits(size) => str += &format!("b{}", size),
            Type::Array(t, len) => str += &format!("[{}; {len}]", self.type_name(t)),
            Type::Unit => str += "()",
            Type::Slice(t) => str += &format!("&[{}]", self.type_name(t)),
            Type::Placeholder => str += "{placeholder}",
        }
        str
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
}
