use super::{Origin, GenericTy, Type, TypeID, TypeIDed, UInstruction, UProgram, UStruct, UVar};
use crate::common::{CompilerMsg, CompilerOutput};
use std::{collections::HashMap, ops::BitOrAssign};

impl UProgram {
    pub fn resolve(&mut self, output: &mut CompilerOutput) {
        let mut unfinished = Vec::new();
        let mut unfinished_new = Vec::new();
        let data = &mut ResData {
            changed: false,
            types: &mut self.types,
            vars: &self.vars,
            structs: &self.structs,
            errs: Vec::new(),
        };
        for f in &self.fns {
            for i in f.flat_iter() {
                if resolve_instr(data, &i.i).unfinished() {
                    unfinished.push(i);
                }
            }
        }
        while !unfinished.is_empty() && data.changed {
            data.changed = false;
            for &i in &unfinished {
                if resolve_instr(data, &i.i).unfinished() {
                    unfinished_new.push(i);
                }
            }
            std::mem::swap(&mut unfinished, &mut unfinished_new);
            unfinished_new.clear();
        }
        for err in &data.errs {
            match err {
                &ResErr::Type {
                    dst,
                    src,
                    ref errs,
                    origin,
                } => {
                    let mut msg = type_assign_err(self, dst, src);
                    for inner in errs {
                        if inner.dst != dst && inner.src != src {
                            msg.push_str("\n    ");
                            msg.push_str(&type_assign_err(self, inner.dst, inner.src));
                        }
                    }
                    output.err(CompilerMsg::new(msg, origin));
                }
                &ResErr::NotCallable { origin, ty } => {
                    output.err(CompilerMsg::new(
                        format!("Cannot call type {}", self.type_name(ty)),
                        origin,
                    ));
                }
            }
        }
        for var in &self.vars {
            match &self.types[var.ty] {
                Type::Error => output.err(CompilerMsg::new(
                    format!("Var {:?} is error type!", var.name),
                    var.origin,
                )),
                Type::Infer => output.err(CompilerMsg::new(
                    format!("Type of {:?} cannot be inferred", var.name),
                    var.origin,
                )),
                Type::Unres(_) => output.err(CompilerMsg::new(
                    format!("Var {:?} type still unresolved!", var.name),
                    var.origin,
                )),
                _ => (),
            }
        }
    }
}

pub fn type_assign_err(p: &mut UProgram, dst: TypeID, src: TypeID) -> String {
    format!(
        "Cannot assign type {} to {}",
        p.type_name(src),
        p.type_name(dst)
    )
}

pub fn resolve_instr(data: &mut ResData, i: &UInstruction) -> InstrRes {
    let mut uf = InstrRes::Finished;
    match &i {
        UInstruction::Call { dst, f, args } => {
            let ftyid = data.vars[f.id].ty;
            let fty = &data.types[ftyid];
            let Type::Fn { args: fargs, ret } = fty else {
                data.errs.push(ResErr::NotCallable {
                    origin: f.origin,
                    ty: ftyid,
                });
                return InstrRes::Finished;
            };
            uf |= data.match_types(dst, ret, dst.origin);
            for (src, dest) in args.iter().zip(fargs) {
                uf |= data.match_types(dest, src, src.origin);
            }
        }
        UInstruction::Mv { dst, src } => {
            uf |= data.match_types(dst, src, src.origin);
        }
        UInstruction::Ref { dst, src } => {
            let Type::Ref(dest_ty) = data.types[data.vars[dst.id].ty] else {
                // TODO: this is probably a compiler error / should never happen
                panic!("how could this happen to me (you)");
            };
            uf |= data.match_types(dest_ty, src, src.origin);
        }
        UInstruction::LoadData { dst, src } => {
            // TODO
        }
        UInstruction::LoadSlice { dst, src } => {
            // TODO
        }
        UInstruction::LoadFn { dst, src } => {
            // TODO
        }
        UInstruction::AsmBlock { instructions, args } => {
            // TODO
        }
        UInstruction::Ret { .. } => {}
        UInstruction::Construct { dst, fields } => {
            let dest_ty = get(vars, dst.id)?;
            let Type::Struct(sty) = dest_ty else {};
            let id = sty.id;
            let Some(struc) = get(id) else {};
            let mut new = HashMap::new();
            for (name, field) in &struc.fields {
                let Some(src) = fields.get(name) else {
                    continue;
                };
                let src_ty = get(vars, src.id)?;
                if let Some(ty) = match_types(vars, types, &field.ty, src_ty) {
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
            set(vars, dst.id, Type::Struct(GenericTy { id, args }));
        }
        UInstruction::If { cond, body } => {
            for i in body {
                uf |= resolve_instr(data, &i.i);
            }
        }
        UInstruction::Loop { body } => {}
        UInstruction::Break => {}
        UInstruction::Continue => {}
    }
    uf
}

pub fn match_types<T1: TypeIDed, T2: TypeIDed>(
    data: &mut TypeResData,
    dst: T1,
    src: T2,
) -> MatchRes {
    let dst = dst.type_id(data.vars);
    let src = src.type_id(data.vars);
    if dst == src {
        return MatchRes::Finished;
    }
    let error = || MatchRes::Error(vec![TypeMismatch { dst, src }]);
    match (&data.types[dst], &data.types[src]) {
        (Type::Error, _) | (_, Type::Error) => MatchRes::Finished,
        (Type::Placeholder, _) | (_, Type::Placeholder) => MatchRes::Unfinished,
        (Type::Infer, Type::Infer) => MatchRes::Unfinished,
        (Type::Infer, x) => {
            *data.changed = true;
            data.types[dst] = x.clone();
            MatchRes::Finished
        }
        (x, Type::Infer) => {
            *data.changed = true;
            data.types[src] = x.clone();
            MatchRes::Finished
        }
        (Type::Struct(dest), Type::Struct(src)) => {
            if dest.id != src.id {
                return error();
            }
            let mut finished = true;
            let mut errors = Vec::new();
            let dargs = dest.args.clone();
            let sargs = dest.args.clone();
            for (darg, sarg) in dargs.iter().zip(&sargs) {
                match match_types(data, darg, sarg) {
                    MatchRes::Unfinished => finished = false,
                    MatchRes::Error(errs) => errors.extend(errs),
                    MatchRes::Finished => (),
                }
            }
            if finished {
                if errors.is_empty() {
                    MatchRes::Finished
                } else {
                    MatchRes::Error(errors)
                }
            } else {
                MatchRes::Unfinished
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
            MatchRes::Finished
        }
        (&Type::Ref(dest), &Type::Ref(src)) => match_types(data, dest, src),
        (&Type::Slice(dest), &Type::Slice(src)) => match_types(data, dest, src),
        (&Type::Array(dest, dlen), &Type::Array(src, slen)) => {
            if dlen == slen {
                match_types(data, dest, src)
            } else {
                error()
            }
        }
        _ => error(),
    }
}

struct ResData<'a> {
    changed: bool,
    types: &'a mut [Type],
    vars: &'a [UVar],
    structs: &'a [UStruct],
    errs: Vec<ResErr>,
}

struct TypeResData<'a> {
    changed: &'a mut bool,
    types: &'a mut [Type],
    vars: &'a [UVar],
    structs: &'a [UStruct],
}

enum ResErr {
    NotCallable {
        origin: Origin,
        ty: TypeID,
    },
    Type {
        dst: TypeID,
        src: TypeID,
        errs: Vec<TypeMismatch>,
        origin: Origin,
    },
}

impl<'a> ResData<'a> {
    pub fn match_types<T1: TypeIDed, T2: TypeIDed>(
        &'a mut self,
        dst: T1,
        src: T2,
        origin: Origin,
    ) -> InstrRes {
        let dst = dst.type_id(self.vars);
        let src = src.type_id(self.vars);
        let res = match_types(
            &mut TypeResData {
                changed: &mut self.changed,
                types: self.types,
                vars: self.vars,
                structs: self.structs,
            },
            dst,
            src,
        );
        match res {
            MatchRes::Unfinished => InstrRes::Unfinished,
            MatchRes::Finished => InstrRes::Finished,
            MatchRes::Error(es) => {
                self.errs.push(ResErr::Type {
                    errs: es,
                    origin,
                    dst,
                    src,
                });
                InstrRes::Finished
            }
        }
    }
}

pub struct TypeMismatch {
    dst: TypeID,
    src: TypeID,
}

pub enum MatchRes {
    Unfinished,
    Finished,
    Error(Vec<TypeMismatch>),
}

pub enum InstrRes {
    Finished,
    Unfinished,
}

impl BitOrAssign for InstrRes {
    fn bitor_assign(&mut self, rhs: Self) {
        match rhs {
            InstrRes::Finished => (),
            InstrRes::Unfinished => *self = InstrRes::Unfinished,
        }
    }
}

impl InstrRes {
    pub fn unfinished(&self) -> bool {
        match self {
            Self::Finished => false,
            Self::Unfinished => true,
        }
    }
}
