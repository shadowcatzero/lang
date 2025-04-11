use super::{Len, StructID, UInstruction, UProgram, UVar, VarID};

#[derive(Clone, PartialEq)]
pub enum Type {
    Bits(u32),
    Struct { id: StructID, args: Vec<Type> },
    Fn { args: Vec<Type>, ret: Box<Type> },
    Ref(Box<Type>),
    Slice(Box<Type>),
    Array(Box<Type>, Len),
    Infer,
    Error,
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
}

impl UProgram {
    pub fn resolve_types(&mut self) {
        // I LOVE RUST
        let mut vars = self.vars.clone();
        for f in self.fns.iter().flatten() {
            for i in &f.instructions {
                self.resolve_instr_types(&mut vars, &i.i);
            }
        }
        self.vars = vars;
    }

    pub fn resolve_instr_types(&self, vars: &mut Vec<Option<UVar>>, i: &UInstruction) {
        match &i {
            UInstruction::Call { dest, f, args } => {
                let ret = self.get_fn_var(f.id).expect("bruh").ret.clone();
                vars[dest.id.0].as_mut().expect("bruh").ty = ret;
            }
            UInstruction::Mv { dest, src } => {
                let dest_ty = get(vars, dest.id);
                let src_ty = get(vars, src.id);
                if let Some(ty) = match_types(dest_ty, src_ty) {
                    set(vars, dest.id, ty.clone());
                    set(vars, src.id, ty);
                }
            }
            UInstruction::Ref { dest, src } => {
                let dest_ty = get(vars, dest.id);
                let src_ty = get(vars, src.id);
                if let Type::Ref(dest_ty) = dest_ty {
                    if let Some(ty) = match_types(dest_ty, src_ty) {
                        set(vars, dest.id, ty.clone().rf());
                        set(vars, src.id, ty);
                    }
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
                // TODO
            }
            UInstruction::If { cond, body } => {
                for i in body {
                    self.resolve_instr_types(vars, &i.i);
                }
            }
            UInstruction::Loop { body } => {
                for i in body {
                    self.resolve_instr_types(vars, &i.i);
                }
            }
            UInstruction::Break => {}
            UInstruction::Continue => {}
        }
    }
}

pub fn get(vars: &[Option<UVar>], id: VarID) -> &Type {
    &vars[id.0]
        .as_ref()
        .expect("PARTIAL BORROWING WOULD BE REALLY COOL")
        .ty
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
        (Type::Error, x) | (x, Type::Error) => None,
        (Type::Infer, x) | (x, Type::Infer) => Some(x.clone()),
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
