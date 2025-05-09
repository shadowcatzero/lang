use std::collections::HashMap;

use super::*;

pub fn inst_fn_var(
    fi: FnInst,
    fns: &[UFunc],
    origin: Origin,
    vars: &mut Vec<UVar>,
    types: &mut Vec<Type>,
) -> VarID {
    let name = fns[fi.id].name.clone();
    let ty = push_id(types, Type::FnInst(fi));
    push_id(
        vars,
        UVar {
            name,
            origin,
            ty: VarTy::Res(ty),
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
            ty: VarTy::Res(ty),
            parent: None,
            children: HashMap::new(),
        },
    );
    id
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
    if gmap.len() == 0 {
        return id;
    }
    match inst_type_(id, types, gmap) {
        Some(new) => new,
        None => id,
    }
}

fn inst_type_(
    id: TypeID,
    types: &mut Vec<Type>,
    gmap: &HashMap<GenericID, TypeID>,
) -> Option<TypeID> {
    let ty = match types[id].clone() {
        Type::Bits(_) => return None,
        Type::Struct(struct_ty) => Type::Struct(StructInst {
            id: struct_ty.id,
            gargs: inst_all(&struct_ty.gargs, types, gmap)?,
        }),
        Type::FnInst(fn_ty) => Type::FnInst(FnInst {
            id: fn_ty.id,
            gargs: inst_all(&fn_ty.gargs, types, gmap)?,
        }),
        Type::Ref(id) => Type::Ref(inst_type_(id, types, gmap)?),
        Type::Slice(id) => Type::Slice(inst_type_(id, types, gmap)?),
        Type::Array(id, len) => Type::Array(inst_type_(id, types, gmap)?, len),
        Type::Unit => return None,
        Type::Generic(gid) => return gmap.get(&gid).map(|id| Some(*id)).unwrap_or_else(|| None),
        Type::Infer => Type::Infer,
        Type::Deref(id) => Type::Deref(inst_type_(id, types, gmap)?),
        Type::Ptr(id) => Type::Ptr(inst_type_(id, types, gmap)?),
        Type::Error => return None,
    };
    Some(push_id(types, ty))
}

fn inst_all(
    ids: &[TypeID],
    types: &mut Vec<Type>,
    gmap: &HashMap<GenericID, TypeID>,
) -> Option<Vec<TypeID>> {
    let mut vec = None;
    for (i, &id) in ids.iter().enumerate() {
        if let Some(id) = inst_type_(id, types, gmap) {
            vec.get_or_insert_with(|| ids.iter().take(i).cloned().collect::<Vec<_>>())
                .push(id);
        } else if let Some(vec) = &mut vec {
            vec.push(id)
        }
    }
    vec
}
