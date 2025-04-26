use std::collections::HashMap;

use super::{assoc::NamePath, GenericID, Len, StructID, UInstruction, UProgram, UVar, VarID};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct MemberRef {
    pub parent: VarID,
    pub name: String,
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct StructTy {
    pub id: StructID,
    pub args: Vec<Type>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Bits(u32),
    Struct(StructTy),
    Generic { id: GenericID },
    Fn { args: Vec<Type>, ret: Box<Type> },
    Ref(Box<Type>),
    Slice(Box<Type>),
    Array(Box<Type>, Len),
    Member(MemberRef),
    Module(NamePath),
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
        // set type of vars referring to functions
        for (i, f) in self.iter_fns() {
            let vi = self.fn_var.var(i);
            vars[vi.0].as_mut().expect("bruh").ty = f.ty(self);
        }
        for (i, f) in self.iter_fns() {
            let mut redo_iter = Vec::new();
            let mut redo_new = Vec::new();
            for i in f.flat_iter() {
                if self.resolve_instr_types(&mut vars, &i.i).is_none() {
                    redo_iter.push(i);
                }
            }
            while !redo_iter.is_empty() {
                for &i in &redo_iter {
                    if self.resolve_instr_types(&mut vars, &i.i).is_none() {
                        redo_new.push(i);
                    }
                }
                std::mem::swap(&mut redo_iter, &mut redo_new);
                redo_new.clear();
            }
        }
        self.vars = vars;
    }

    pub fn resolve_instr_types(&self, vars: &mut [Option<UVar>], i: &UInstruction) -> Option<()> {
        'outer: {
            match &i {
                UInstruction::Call { dest, f, args } => {
                    let fun = self.get_fn_var(f.id).expect("bruh");
                    vars[dest.id.0].as_mut().expect("bruh").ty = fun.ret.clone();
                    for (src, &dest) in args.iter().zip(&fun.args) {
                        let dest_ty = get(vars, dest)?;
                        let src_ty = get(vars, src.id)?;
                        if let Some(ty) = self.match_types(dest_ty, src_ty) {
                            set(vars, dest, ty.clone());
                            set(vars, src.id, ty);
                        }
                    }
                }
                UInstruction::Mv { dest, src } => {
                    let dest_ty = get(vars, dest.id)?;
                    let src_ty = get(vars, src.id)?;
                    if let Some(ty) = self.match_types(dest_ty, src_ty) {
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
                    if let Some(ty) = self.match_types(dest_ty, src_ty) {
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
                    let Type::Struct(sty) = dest_ty else {
                        break 'outer;
                    };
                    let id = sty.id;
                    let Some(struc) = self.get(id) else {
                        break 'outer;
                    };
                    let mut new = HashMap::new();
                    for (name, field) in &struc.fields {
                        let Some(src) = fields.get(name) else {
                            continue;
                        };
                        let src_ty = get(vars, src.id)?;
                        if let Some(ty) = self.match_types(&field.ty, src_ty) {
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
                    set(vars, dest.id, Type::Struct(StructTy { id, args }));
                }
                UInstruction::If { cond, body: _ } => {}
                UInstruction::Loop { body: _ } => {}
                UInstruction::Break => {}
                UInstruction::Continue => {}
            }
        }
        Some(())
    }

    pub fn match_types(&self, dest: &Type, src: &Type) -> Option<Type> {
        if dest == src {
            return None;
        }
        match (dest, src) {
            (Type::Error, _) | (_, Type::Error) => None,
            (Type::Placeholder, _) | (_, Type::Placeholder) => None,
            (Type::Infer, x) | (x, Type::Infer) => Some(x.clone()),
            // TODO: handle constraints?
            (Type::Generic { id }, x) | (x, Type::Generic { id }) => Some(x.clone()),
            (Type::Struct(dest), Type::Struct(src)) => {
                if dest.id != src.id {
                    return None;
                }
                let mut args = Vec::new();
                let mut changed = false;
                for (darg, sarg) in dest.args.iter().zip(&src.args) {
                    if let Some(ty) = self.match_types(darg, sarg) {
                        args.push(ty);
                        changed = true;
                    } else if darg != sarg {
                        return None;
                    } else {
                        args.push(darg.clone());
                    }
                }
                if changed {
                    Some(Type::Struct(StructTy { id: dest.id, args }))
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
            (Type::Ref(dest), Type::Ref(src)) => Some(self.match_types(dest, src)?.rf()),
            (Type::Slice(dest), Type::Slice(src)) => Some(self.match_types(dest, src)?.slice()),
            (Type::Array(dest, dlen), Type::Array(src, slen)) => {
                if dlen != slen {
                    return None;
                }
                Some(self.match_types(dest, src)?.arr(*dlen))
            }
            _ => None,
        }
    }
}

pub fn get(vars: &[Option<UVar>], id: VarID) -> Option<&Type> {
    let mut var = vars[id.0]
        .as_ref()
        .expect("PARTIAL BORROWING WOULD BE REALLY COOL");
    if var.ty == Type::Placeholder {
        return None;
    }
    while let Type::Member(m) = &var.ty {
        var = vars[m.parent.0].as_ref().expect("xd");
    }
    //   x.y().z == a.b.c()
    // 0 ------- member(1, z)
    // 1 -----   call(2)
    // 2 ---     member(3, y)
    // 3 -       x
    //
    // 0            ------- call(1)
    // 1            -----   member(c, 2)
    // 2            ---     member(b, 3)
    // 3            -       a
    Some(&var.ty)
}

pub fn set(vars: &mut [Option<UVar>], id: VarID, ty: Type) {
    vars[id.0]
        .as_mut()
        .expect("PARTIAL BORROWING WOULD BE REALLY COOL")
        .ty = ty;
}
