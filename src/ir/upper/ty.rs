use std::collections::HashMap;

use super::{
    push_id, FnID, GenericID, IdentID, Len, Origin, ResErr, StructID, TypeDef, TypeID, UFunc,
    UGeneric, UProgram, UStruct, UVar, VarID,
};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct FieldRef {
    pub parent: VarID,
    pub name: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StructInst {
    pub id: StructID,
    /// assumed to be valid
    pub gargs: Vec<TypeID>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FnInst {
    pub id: FnID,
    /// assumed to be valid
    pub gargs: Vec<TypeID>,
}

#[derive(Clone)]
pub enum Type {
    Bits(u32),
    Struct(StructInst),
    FnRef(FnInst),
    // this can be added for constraints later (F: fn(...) -> ...)
    // Fn { args: Vec<TypeID>, ret: TypeID },
    Ref(TypeID),
    Deref(TypeID),
    Slice(TypeID),
    Array(TypeID, Len),
    Unit,
    // "fake" types
    Unres(IdentID),
    Generic(GenericID),
    Infer,
    Error,
}

impl Type {
    pub fn rf(self, p: &mut UProgram) -> Self {
        p.def_ty(self).rf()
    }
    pub fn derf(self, p: &mut UProgram) -> Self {
        p.def_ty(self).derf()
    }
    pub fn arr(self, p: &mut UProgram, len: Len) -> Self {
        p.def_ty(self).arr(len)
    }
    pub fn slice(self, p: &mut UProgram) -> Self {
        p.def_ty(self).slice()
    }
}

impl TypeID {
    pub fn rf(self) -> Type {
        Type::Ref(self)
    }
    pub fn derf(self) -> Type {
        Type::Deref(self)
    }
    pub fn arr(self, len: Len) -> Type {
        Type::Array(self, len)
    }
    pub fn slice(self) -> Type {
        Type::Slice(self)
    }
}

impl Type {
    pub fn bx(self) -> Box<Self> {
        Box::new(self)
    }
}

pub fn inst_fn_var(
    fi: FnInst,
    fns: &[UFunc],
    origin: Origin,
    vars: &mut Vec<UVar>,
    types: &mut Vec<Type>,
) -> VarID {
    let name = fns[fi.id].name.clone();
    let ty = push_id(types, Type::FnRef(fi));
    push_id(
        vars,
        UVar {
            name,
            origin,
            ty,
            parent: None,
            children: HashMap::new(),
        },
    )
}

pub fn inst_struct_var(
    si: StructInst,
    structs: &[UStruct],
    origin: Origin,
    vars: &mut Vec<UVar>,
    types: &mut Vec<Type>,
) -> VarID {
    let name = structs[si.id].name.clone();
    let ty = push_id(types, Type::Struct(si));
    let id = push_id(
        vars,
        UVar {
            name,
            origin,
            ty,
            parent: None,
            children: HashMap::new(),
        },
    );
    inst_var(vars, structs, id, types);
    id
}

pub fn inst_var(vars: &mut Vec<UVar>, structs: &[UStruct], id: VarID, types: &mut Vec<Type>) {
    match &types[resolve_refs(types, vars[id].ty)] {
        Type::Struct(si) => {
            let fields = &structs[si.id].fields;
            let s_gargs = &structs[si.id].gargs;
            let gmap = inst_gmap(s_gargs, &si.gargs);
            let children = fields
                .iter()
                .map(|(name, f)| {
                    (name.clone(), {
                        let ty = inst_type(f.ty, types, &gmap);
                        let fid = push_id(
                            vars,
                            UVar {
                                name: name.clone(),
                                origin: f.origin,
                                ty,
                                parent: Some(id),
                                children: HashMap::new(),
                            },
                        );
                        inst_var(vars, structs, fid, types);
                        fid
                    })
                })
                .collect();
            vars[id].children = children;
        }
        _ => (),
    }
}

pub fn resolve_refs(types: &[Type], id: TypeID) -> TypeID {
    if let Type::Deref(rid) = types[id]
        && let Type::Ref(nid) = types[rid]
    {
        nid
    } else {
        id
    }
}

pub fn validate_gargs(
    dst: &[GenericID],
    src: &[TypeID],
    generics: &[UGeneric],
    types: &[Type],
    errs: &mut Vec<ResErr>,
    origin: Origin,
) -> Result<(), Option<ResErr>> {
    if dst.len() != src.len() {
        return Err(Some(ResErr::GenericCount {
            origin,
            expected: dst.len(),
            found: src.len(),
        }));
    }
    for (dst, src) in dst.iter().zip(src.iter()) {
        let g = &generics[dst];
        let t = &types[src];
        // TODO: validate trait constraints
    }
    Ok(())
}

/// gargs assumed to be valid
pub fn inst_typedef(def: &TypeDef, gargs: &[TypeID], types: &mut Vec<Type>) -> TypeID {
    let gmap = inst_gmap(&def.gargs, &gargs);
    inst_type(def.ty, types, &gmap)
}

pub fn inst_gmap(dst: &[GenericID], src: &[TypeID]) -> HashMap<GenericID, TypeID> {
    let mut gmap = HashMap::new();
    for (&gid, &tid) in dst.iter().zip(src) {
        gmap.insert(gid, tid);
    }
    gmap
}

pub fn inst_type(id: TypeID, types: &mut Vec<Type>, gmap: &HashMap<GenericID, TypeID>) -> TypeID {
    let ty = match types[id].clone() {
        Type::Bits(_) => return id,
        Type::Struct(struct_ty) => Type::Struct(StructInst {
            id: struct_ty.id,
            gargs: struct_ty
                .gargs
                .iter()
                .map(|id| inst_type(*id, types, gmap))
                .collect(),
        }),
        Type::FnRef(fn_ty) => Type::FnRef(FnInst {
            id: fn_ty.id,
            gargs: fn_ty
                .gargs
                .iter()
                .map(|id| inst_type(*id, types, gmap))
                .collect(),
        }),
        Type::Ref(id) => Type::Ref(inst_type(id, types, gmap)),
        Type::Deref(id) => Type::Deref(inst_type(id, types, gmap)),
        Type::Slice(id) => Type::Slice(inst_type(id, types, gmap)),
        Type::Array(id, len) => Type::Array(inst_type(id, types, gmap), len),
        Type::Unit => Type::Unit,
        Type::Unres(mod_path) => Type::Unres(mod_path.clone()),
        Type::Generic(gid) => return gmap.get(&gid).cloned().unwrap_or_else(|| id),
        Type::Infer => Type::Infer,
        Type::Error => Type::Error,
    };
    push_id(types, ty)
}

// type Test<T, U> = (&T, &U)
