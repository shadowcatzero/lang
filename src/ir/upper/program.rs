use std::{collections::HashMap, fmt::Debug};

use super::{inst::VarInst, *};

pub struct UProgram {
    pub fns: Vec<Option<UFunc>>,
    pub vars: Vec<Option<UVar>>,
    pub structs: Vec<Option<UStruct>>,
    pub data: Vec<Option<UData>>,
    pub start: Option<FnID>,
    pub names: NameMap,
    // todo: these feel weird raw
    pub fn_map: HashMap<VarID, FnID>,
    pub inv_fn_map: Vec<VarID>,
    pub temp: usize,
    pub name_stack: Vec<HashMap<String, Idents>>,
}

pub struct NameMap {
    names: [Vec<String>; NAMED_KINDS],
    inv_names: [HashMap<String, usize>; NAMED_KINDS],
}

impl NameMap {
    pub fn new() -> Self {
        Self {
            names: core::array::from_fn(|_| Vec::new()),
            inv_names: core::array::from_fn(|_| HashMap::new()),
        }
    }
    pub fn get<K: Kind>(&self, id: ID<K>) -> &str {
        &self.names[K::INDEX][id.0]
    }
    pub fn lookup<K: Kind>(&self, name: &str) -> Option<ID<K>> {
        Some(ID::new(*self.inv_names[K::INDEX].get(name)?))
    }
    pub fn push<K: Kind>(&mut self, name: String) {
        self.inv_names[K::INDEX].insert(name.clone(), self.names[K::INDEX].len());
        self.names[K::INDEX].push(name);
    }
}

impl UProgram {
    pub fn new() -> Self {
        Self {
            fns: Vec::new(),
            vars: Vec::new(),
            structs: Vec::new(),
            data: Vec::new(),
            start: None,
            names: NameMap::new(),
            fn_map: HashMap::new(),
            inv_fn_map: Vec::new(),
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
        self.fns[self.fn_map.get(&id)?.0].as_ref()
    }
    pub fn size_of_type(&self, ty: &Type) -> Option<Size> {
        // TODO: target matters
        Some(match ty {
            Type::Bits(b) => *b,
            Type::Struct { id, args } => self.structs[id.0]
                .as_ref()?
                .fields
                .iter()
                .try_fold(0, |sum, f| Some(sum + self.size_of_type(&f.ty)?))?,
            Type::Fn { args, ret } => todo!(),
            Type::Ref(_) => 64,
            Type::Array(ty, len) => self.size_of_type(ty)? * len,
            Type::Slice(_) => 128,
            Type::Infer => return None,
            Type::Error => return None,
            Type::Unit => 0,
        })
    }
    pub fn size_of_var(&self, var: VarID) -> Option<Size> {
        self.size_of_type(&self.get(var)?.ty)
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
            Some(UVar { parent, origin, ty }),
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

    pub fn def<K: Kind>(&mut self, name: String, k: Option<K>) -> ID<K> {
        self.names.push::<K>(name);
        let vec = K::from_program_mut(self);
        let id = ID::new(vec.len());
        vec.push(k);
        id
    }

    pub fn def_searchable<K: Kind>(&mut self, name: String, k: Option<K>) -> ID<K> {
        let id = self.def(name.clone(), k);
        self.name_on_stack(id, name);
        id
    }

    pub fn type_name(&self, ty: &Type) -> String {
        let mut str = String::new();
        match ty {
            Type::Struct { id: base, args } => {
                str += self.names.get(*base);
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
            Type::Bits(size) => str += &format!("b{}", size),
            Type::Array(t, len) => str += &format!("[{}; {len}]", self.type_name(t)),
            Type::Unit => str += "()",
            Type::Slice(t) => str += &format!("&[{}]", self.type_name(t)),
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
    pub fn var_offset(&self, var: VarID) -> Option<VarOffset> {
        let mut current = VarOffset { id: var, offset: 0 };
        while let Some(parent) = self.get(current.id)?.parent {
            current.id = parent.var;
            current.offset += self.field_offset(parent.struc, parent.field)?;
        }
        Some(current)
    }
    pub fn field_offset(&self, struct_id: StructID, field: FieldID) -> Option<Len> {
        let struc = self.get(struct_id)?;
        let mut offset = 0;
        for i in 0..field.0 {
            offset += self.size_of_type(&struc.fields[i].ty)?;
        }
        Some(offset)
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

#[derive(Debug, Clone, Copy)]
pub struct Ident {
    id: usize,
    kind: usize,
}

impl<K: Kind> From<ID<K>> for Ident {
    fn from(id: ID<K>) -> Self {
        Self {
            id: id.0,
            kind: K::INDEX,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Idents {
    pub latest: Ident,
    pub kinds: [Option<usize>; NAMED_KINDS],
}

impl Idents {
    fn new(latest: Ident) -> Self {
        let mut s = Self {
            latest,
            kinds: [None; NAMED_KINDS],
        };
        s.insert(latest);
        s
    }
    fn insert(&mut self, i: Ident) {
        self.latest = i;
        self.kinds[i.kind] = Some(i.id);
    }
    pub fn get<K: Kind>(&self) -> Option<ID<K>> {
        self.kinds[K::INDEX].map(|i| i.into())
    }
}
