use std::{collections::HashMap, fmt::Debug};

use super::{inst::VarInst, *};

pub struct IRUProgram {
    pub fn_defs: Vec<FnDef>,
    pub var_defs: Vec<VarDef>,
    pub struct_defs: Vec<StructDef>,
    pub data_defs: Vec<DataDef>,
    pub fns: Vec<Option<IRUFunction>>,
    pub data: Vec<Vec<u8>>,
    pub fn_map: HashMap<VarID, FnID>,
    pub temp: usize,
    pub stack: Vec<HashMap<String, Idents>>,
}

impl IRUProgram {
    pub fn new() -> Self {
        Self {
            fn_defs: Vec::new(),
            var_defs: Vec::new(),
            struct_defs: Vec::new(),
            data_defs: Vec::new(),
            data: Vec::new(),
            fn_map: HashMap::new(),
            fns: Vec::new(),
            temp: 0,
            stack: vec![HashMap::new()],
        }
    }
    pub fn push(&mut self) {
        self.stack.push(HashMap::new());
    }
    pub fn pop(&mut self) {
        self.stack.pop();
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
    pub fn get_var(&self, id: VarID) -> &VarDef {
        &self.var_defs[id.0]
    }
    pub fn get_fn(&self, id: FnID) -> &FnDef {
        &self.fn_defs[id.0]
    }
    pub fn get_data(&self, id: DataID) -> &DataDef {
        &self.data_defs[id.0]
    }
    pub fn get_fn_var(&self, id: VarID) -> Option<&FnDef> {
        Some(&self.fn_defs[self.fn_map.get(&id)?.0])
    }
    pub fn get_struct(&self, id: StructID) -> &StructDef {
        &self.struct_defs[id.0]
    }
    pub fn alias_fn(&mut self, name: &str, id: FnID) {
        self.insert(name, Ident::Fn(id));
    }
    pub fn named_var(&mut self, def: VarDef) -> VarID {
        // TODO: this is stupid
        let id = self.def_var(def.clone());
        self.name_var(&def, id);
        id
    }
    pub fn name_var(&mut self, def: &VarDef, var: VarID) {
        self.insert(&def.name, Ident::Var(var));
    }
    pub fn def_var(&mut self, var: VarDef) -> VarID {
        let i = self.var_defs.len();
        self.var_defs.push(var);
        VarID(i)
    }
    pub fn size_of_type(&self, ty: &Type) -> Option<Size> {
        // TODO: target matters
        Some(match ty {
            Type::Bits(b) => *b,
            Type::Struct { id, args } => self.struct_defs[id.0]
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
    pub fn struct_layout() {}
    pub fn size_of_var(&self, var: VarID) -> Option<Size> {
        self.size_of_type(&self.var_defs[var.0].ty)
    }
    pub fn temp_subvar(&mut self, origin: Origin, ty: Type, parent: FieldRef) -> VarInst {
        self.temp_var_inner(origin, ty, Some(parent))
    }
    pub fn temp_var(&mut self, origin: Origin, ty: Type) -> VarInst {
        self.temp_var_inner(origin, ty, None)
    }

    fn temp_var_inner(&mut self, origin: Origin, ty: Type, parent: Option<FieldRef>) -> VarInst {
        let v = self.def_var(VarDef {
            name: format!("temp{}", self.temp),
            parent,
            origin,
            ty,
        });
        self.temp += 1;
        VarInst {
            id: v,
            span: origin,
        }
    }

    pub fn def_fn(&mut self, def: FnDef) -> FnID {
        let i = self.fn_defs.len();
        let id = FnID(i);
        let var_def = VarDef {
            name: def.name.clone(),
            parent: None,
            origin: def.origin,
            ty: def.ty(),
        };

        let vid = self.def_var(var_def);
        self.insert(&def.name, Ident::Var(vid));
        self.fn_map.insert(vid, id);

        self.insert(&def.name, Ident::Fn(id));
        self.fn_defs.push(def);
        self.fns.push(None);

        id
    }
    pub fn def_struct(&mut self, def: StructDef) -> StructID {
        let i = self.struct_defs.len();
        let id = StructID(i);
        self.insert(&def.name, Ident::Type(id));
        self.struct_defs.push(def);
        id
    }
    pub fn def_data(&mut self, def: DataDef, bytes: Vec<u8>) -> DataID {
        let i = self.data.len();
        self.data_defs.push(def);
        self.data.push(bytes);
        DataID(i)
    }
    pub fn type_name(&self, ty: &Type) -> String {
        let mut str = String::new();
        match ty {
            Type::Struct { id: base, args } => {
                str += &self.get_struct(*base).name;
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
    fn insert(&mut self, name: &str, id: Ident) {
        let idx = self.stack.len() - 1;
        let last = &mut self.stack[idx];
        if let Some(l) = last.get_mut(name) {
            l.insert(id);
        } else {
            last.insert(name.to_string(), Idents::new(id));
        }
    }
    pub fn write_fn(&mut self, id: FnID, f: IRUFunction) {
        self.fns[id.0] = Some(f);
    }
    pub fn iter_vars(&self) -> impl Iterator<Item = (VarID, &VarDef)> {
        self.var_defs.iter().enumerate().map(|(i, v)| (VarID(i), v))
    }
    pub fn iter_fns(&self) -> impl Iterator<Item = (FnID, &IRUFunction)> {
        self.fns
            .iter()
            .enumerate()
            .flat_map(|(i, f)| Some((FnID(i), f.as_ref()?)))
    }
    pub fn var_offset(&self, var: VarID) -> Option<VarOffset> {
        let mut current = VarOffset { id: var, offset: 0 };
        while let Some(parent) = self.var_defs[current.id.0].parent {
            current.id = parent.var;
            current.offset += self.field_offset(parent.struc, parent.field)?;
        }
        Some(current)
    }
    pub fn field_offset(&self, struct_id: StructID, field: FieldID) -> Option<Len> {
        let struc = self.get_struct(struct_id);
        let mut offset = 0;
        for i in 0..field.0 {
            offset += self.size_of_type(&struc.fields[i].ty)?;
        }
        Some(offset)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Ident {
    Var(VarID),
    Fn(FnID),
    Type(StructID),
}

#[derive(Debug, Clone, Copy)]
pub struct Idents {
    pub latest: Ident,
    pub var: Option<VarID>,
    pub func: Option<FnID>,
    pub struc: Option<StructID>,
}

impl Idents {
    fn new(latest: Ident) -> Self {
        let mut s = Self {
            latest,
            var: None,
            func: None,
            struc: None,
        };
        s.insert(latest);
        s
    }
    fn insert(&mut self, i: Ident) {
        self.latest = i;
        match i {
            Ident::Var(v) => {
                self.var = Some(v);
            }
            Ident::Fn(f) => {
                self.func = Some(f);
            }
            Ident::Type(t) => self.struc = Some(t),
        }
    }
}
