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
    let ty = push_id(types, RType::FnRef(fi).ty());
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
    let ty = push_id(types, RType::Struct(si).ty());
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
    match real_type(types, vars[id].ty) {
        RType::Struct(si) => {
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
        Type::Real(rty) => match rty {
            RType::Bits(_) => return None,
            RType::Struct(struct_ty) => RType::Struct(StructInst {
                id: struct_ty.id,
                gargs: inst_all(&struct_ty.gargs, types, gmap)?,
            }),
            RType::FnRef(fn_ty) => RType::FnRef(FnInst {
                id: fn_ty.id,
                gargs: inst_all(&fn_ty.gargs, types, gmap)?,
            }),
            RType::Ref(id) => RType::Ref(inst_type_(id, types, gmap)?),
            RType::Slice(id) => RType::Slice(inst_type_(id, types, gmap)?),
            RType::Array(id, len) => RType::Array(inst_type_(id, types, gmap)?, len),
            RType::Unit => return None,
            RType::Generic(gid) => {
                return gmap
                    .get(&gid)
                    .map(|id| Some(*id))
                    .unwrap_or_else(|| None)
            }
            RType::Infer => RType::Infer,
        }
        .ty(),
        Type::Deref(id) => Type::Deref(inst_type_(id, types, gmap)?),
        Type::Ptr(id) => Type::Ptr(inst_type_(id, types, gmap)?),
        Type::Unres(mod_path) => Type::Unres(mod_path.clone()),
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

