use std::collections::{HashMap, HashSet};

use super::{GenericID, Len, StructID, UInstruction, UProgram, UVar, VarID};

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Bits(u32),
    Struct { id: StructID, args: Vec<Type> },
    Generic { id: GenericID },
    Fn { args: Vec<Type>, ret: Box<Type> },
    Ref(Box<Type>),
    Slice(Box<Type>),
    Array(Box<Type>, Len),
    Infer,
    Error,
    Placeholder,
    Unit,
}

impl Type {
    pub fn rf(self) -> Self {
        Self::Ref(Box::new(self))
    }
    pub fn arr(self, len: Len) -> Self {
        Self::Array(Box::new(self), len)
    }
    pub fn slice(self) -> Self {
        Self::Slice(Box::new(self))
    }
    pub fn is_real(&self) -> bool {
        !matches!(self, Self::Error | Self::Placeholder | Self::Infer)
    }
}

impl UProgram {
    pub fn resolve_types(&mut self) {
        // I LOVE RUST
        let mut vars = self.vars.clone();
        for (i, f) in self.iter_fns() {
            let mut redo_iter = Vec::new();
            let mut ph_vars = Vec::new();
            let mut redo_new = Vec::new();
            for i in f.flat_iter() {
                if let Err(id) = self.resolve_instr_types(&mut vars, &i.i) {
                    redo_iter.push(i);
                    ph_vars.push(id);
                }
            }
            while !redo_iter.is_empty() {
                let mut new_ph = Vec::new();
                for id in &ph_vars {
                    let i = id.0;
                    let Some(var) = &vars[i] else {
                        continue;
                    };
                    if var.ty == Type::Placeholder
                        && let Some(parent) = var.parent.as_ref()
                    {
                        let pty = &vars[parent.var.0].as_ref().unwrap().ty;
                        if let Some(ft) = self.field_type(pty, &parent.field) {
                            vars[i].as_mut().unwrap().ty = ft.clone();
                        } else {
                            new_ph.push(parent.var);
                        }
                    }
                }
                ph_vars = new_ph;
                for &i in &redo_iter {
                    if let Err(id) = self.resolve_instr_types(&mut vars, &i.i) {
                        redo_new.push(i);
                        ph_vars.push(id);
                    }
                }
                std::mem::swap(&mut redo_iter, &mut redo_new);
                redo_new.clear();
            }
        }
        self.vars = vars;
    }

    pub fn resolve_instr_types(
        &self,
        vars: &mut Vec<Option<UVar>>,
        i: &UInstruction,
    ) -> Result<(), VarID> {
        'outer: {
            match &i {
                UInstruction::Call { dest, f, args } => {
                    let fun = self.get_fn_var(f.id).expect("bruh");
                    vars[dest.id.0].as_mut().expect("bruh").ty = fun.ret.clone();
                    for (src, &dest) in args.iter().zip(&fun.args) {
                        let dest_ty = get(vars, dest)?;
                        let src_ty = get(vars, src.id)?;
                        if let Some(ty) = match_types(dest_ty, src_ty) {
                            set(vars, dest, ty.clone());
                            set(vars, src.id, ty);
                        }
                    }
                }
                UInstruction::Mv { dest, src } => {
                    let dest_ty = get(vars, dest.id)?;
                    let src_ty = get(vars, src.id)?;
                    if let Some(ty) = match_types(dest_ty, src_ty) {
                        set(vars, dest.id, ty.clone());
                        set(vars, src.id, ty);
                    }
                }
                UInstruction::Ref { dest, src } => {
                    let dest_ty = get(vars, dest.id)?;
                    let src_ty = get(vars, src.id)?;
                    let Type::Ref(dest_ty) = dest_ty else {
                        break 'outer;
                    };
                    if let Some(ty) = match_types(dest_ty, src_ty) {
                        set(vars, dest.id, ty.clone().rf());
                        set(vars, src.id, ty);
                    }
                }
                UInstruction::LoadData { dest, src } => {
                    // TODO
                }
                UInstruction::LoadSlice { dest, src } => {
                    // TODO
                }
                UInstruction::LoadFn { dest, src } => {
                    // TODO
                }
                UInstruction::AsmBlock { instructions, args } => {
                    // TODO
                }
                UInstruction::Ret { .. } => {}
                UInstruction::Construct { dest, fields } => {
                    let dest_ty = get(vars, dest.id)?;
                    let Type::Struct { id, args } = dest_ty else {
                        break 'outer;
                    };
                    let id = *id;
                    let Some(struc) = self.get(id) else {
                        break 'outer;
                    };
                    let mut new = HashMap::new();
                    for (name, field) in &struc.fields {
                        let Some(src) = fields.get(name) else {
                            continue;
                        };
                        let src_ty = get(vars, src.id)?;
                        if let Some(ty) = match_types(&field.ty, src_ty) {
                            if let Type::Generic { id } = field.ty {
                                new.insert(id, ty.clone());
                            }
                            set(vars, src.id, ty);
                        }
                    }
                    let mut args: Vec<_> = struc
                        .generics
                        .iter()
                        .map(|&id| Type::Generic { id })
                        .collect();
                    for (i, g) in struc.generics.iter().enumerate() {
                        if let Some(ty) = new.remove(g) {
                            args[i] = ty;
                        }
                    }
                    // for arg in &args {
                    //     println!("{:?}", self.type_name(arg));
                    // }
                    set(vars, dest.id, Type::Struct { id, args });
                }
                UInstruction::If { cond, body } => {}
                UInstruction::Loop { body } => {}
                UInstruction::Break => {}
                UInstruction::Continue => {}
            }
        }
        Ok(())
    }
}

pub fn get(vars: &[Option<UVar>], id: VarID) -> Result<&Type, VarID> {
    let var = vars[id.0]
        .as_ref()
        .expect("PARTIAL BORROWING WOULD BE REALLY COOL");
    if var.ty == Type::Placeholder {
        return Err(id);
    }
    Ok(&var.ty)
}

pub fn set(vars: &mut [Option<UVar>], id: VarID, ty: Type) {
    vars[id.0]
        .as_mut()
        .expect("PARTIAL BORROWING WOULD BE REALLY COOL")
        .ty = ty;
}

pub fn match_types(dest: &Type, src: &Type) -> Option<Type> {
    if dest == src {
        return None;
    }
    match (dest, src) {
        (Type::Error, _) | (_, Type::Error) => None,
        (Type::Placeholder, _) | (_, Type::Placeholder) => None,
        (Type::Infer, x) | (x, Type::Infer) => Some(x.clone()),
        // TODO: handle constraints?
        (Type::Generic { id }, x) | (x, Type::Generic { id }) => Some(x.clone()),
        (
            Type::Struct {
                id: dest_id,
                args: dest_args,
            },
            Type::Struct {
                id: src_id,
                args: src_args,
            },
        ) => {
            if dest_id != src_id {
                return None;
            }
            let mut args = Vec::new();
            let mut changed = false;
            for (darg, sarg) in dest_args.iter().zip(src_args) {
                if let Some(ty) = match_types(darg, sarg) {
                    args.push(ty);
                    changed = true;
                } else if darg != sarg {
                    return None;
                } else {
                    args.push(darg.clone());
                }
            }
            if changed {
                Some(Type::Struct { id: *dest_id, args })
            } else {
                None
            }
        }
        (
            Type::Fn {
                args: dest_args,
                ret: dest_ret,
            },
            Type::Fn {
                args: src_args,
                ret: src_ret,
            },
        ) => {
            // TODO
            None
        }
        (Type::Ref(dest), Type::Ref(src)) => Some(match_types(dest, src)?.rf()),
        (Type::Slice(dest), Type::Slice(src)) => Some(match_types(dest, src)?.slice()),
        (Type::Array(dest, dlen), Type::Array(src, slen)) => {
            if dlen != slen {
                return None;
            }
            Some(match_types(dest, src)?.arr(*dlen))
        }
        _ => None,
    }
}
