use std::collections::HashMap;

use super::{
    push_id, FnID, GenericID, Len, ModPath, Origin, ResErr, StructID, TypeID, UFunc, UGeneric,
    UProgram, UStruct, UVar, VarID,
};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct FieldRef {
    pub parent: VarID,
    pub name: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StructInst {
    pub id: StructID,
    pub gargs: Vec<TypeID>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FnInst {
    pub id: FnID,
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
    Unres(ModPath),
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
    fi: &FnInst,
    fns: &[UFunc],
    origin: Origin,
    vars: &mut Vec<UVar>,
    types: &mut Vec<Type>,
    generics: &[UGeneric],
    errs: &mut Vec<ResErr>,
) -> VarID {
    let ty = inst_fn_ty(fi, fns, types, generics, errs);
    let name = fns[fi.id].name.clone();
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

pub fn inst_fn_ty(
    fi: &FnInst,
    fns: &[UFunc],
    types: &mut Vec<Type>,
    generics: &[UGeneric],
    errs: &mut Vec<ResErr>,
) -> TypeID {
    let f = &fns[fi.id];
    let ty = Type::FnRef(FnInst {
        id: fi.id,
        gargs: inst_generics(&f.gargs, &fi.gargs, types, generics, errs),
    });
    push_id(types, ty)
}

pub fn inst_struct_var(
    si: &StructInst,
    structs: &[UStruct],
    origin: Origin,
    vars: &mut Vec<UVar>,
    types: &mut Vec<Type>,
    generics: &[UGeneric],
    errs: &mut Vec<ResErr>,
) -> VarID {
    let ty = inst_struct_ty(si, structs, types, generics, errs);
    let name = structs[si.id].name.clone();
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

pub fn inst_struct_ty(
    si: &StructInst,
    structs: &[UStruct],
    types: &mut Vec<Type>,
    generics: &[UGeneric],
    errs: &mut Vec<ResErr>,
) -> TypeID {
    let s = &structs[si.id];
    let ty = Type::Struct(StructInst {
        id: si.id,
        gargs: inst_generics(&s.gargs, &si.gargs, types, generics, errs),
    });
    push_id(types, ty)
}

pub fn inst_generics(
    source: &[GenericID],
    args: &[TypeID],
    types: &mut Vec<Type>,
    // will be needed when constraints are added
    _generics: &[UGeneric],
    errs: &mut Vec<ResErr>,
) -> Vec<TypeID> {
    if source.len() != args.len() {
        // don't want unequal lengths to be inferred further
        return source.iter().map(|_| push_id(types, Type::Error)).collect();
    }
    let mut gargs = Vec::new();
    let mut gmap = HashMap::new();
    for &gid in source {
        let id = push_id(types, Type::Error);
        gmap.insert(gid, id);
        gargs.push(id);
    }
    for (gid, &ty) in source.iter().zip(args) {
        inst_type_ins(
            |types, ty| {
                let id = gmap[gid];
                types[id] = ty;
                id
            },
            ty,
            types,
            &gmap,
        );
    }
    gargs
}

pub fn inst_type(id: TypeID, types: &mut Vec<Type>, gmap: &HashMap<GenericID, TypeID>) -> TypeID {
    inst_type_ins(push_id, id, types, gmap)
}

pub fn inst_type_ins(
    insert: impl Fn(&mut Vec<Type>, Type) -> TypeID,
    id: TypeID,
    types: &mut Vec<Type>,
    gmap: &HashMap<GenericID, TypeID>,
) -> TypeID {
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
    insert(types, ty)
}
