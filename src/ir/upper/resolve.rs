use super::{
    report_errs, ControlFlowOp, DataID, Kind, MemberTy, Origin, ResErr, Type, TypeID, TypeMismatch, UData, UFunc, UGeneric, UInstrInst, UInstruction, UModule, UProgram, UStruct, UVar, VarID, VarInst, VarInstID, VarStatus
};
use crate::{
    common::{CompilerMsg, CompilerOutput},
    ir::{inst_fn_var, inst_struct_ty, KindTy, ID},
};
use std::{
    collections::HashSet,
    convert::Infallible,
    ops::{BitOrAssign, FromResidual},
};

// dawg this file is way too long

impl UProgram {
    pub fn resolve(&mut self, output: &mut CompilerOutput) {
        let mut unfinished = Vec::new();
        let mut data = ResData {
            unfinished: Vec::new(),
            changed: false,
            types: &mut self.types,
            s: Sources {
                insts: &mut self.vars_insts,
                vars: &mut self.vars,
                fns: &self.fns,
                structs: &self.structs,
                generics: &self.generics,
                data: &self.data,
                modules: &self.modules,
            },
            errs: Vec::new(),
        };
        for (fid, f) in self.fns.iter().enumerate() {
            for i in &f.instructions {
                resolve_instr(
                    &mut data,
                    ResolveCtx {
                        ret: f.ret,
                        breakable: false,
                        i,
                    },
                );
            }
            // this currently works bc expressions create temporary variables
            // although you can't do things like loop {return 3} (need to analyze control flow)
            if !matches!(data.types[f.ret], Type::Unit) {
                if f.instructions
                    .last()
                    .is_none_or(|i| !matches!(i.i, UInstruction::Ret { .. }))
                {
                    data.errs.push(ResErr::NoReturn { fid });
                }
            }
        }
        while !data.unfinished.is_empty() && data.changed {
            data.changed = false;
            std::mem::swap(&mut data.unfinished, &mut unfinished);
            for ctx in unfinished.drain(..) {
                resolve_instr(&mut data, ctx);
            }
        }
        let errs = data.errs;
        report_errs(self, output, errs);
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

#[derive(Clone, Copy)]
struct ResolveCtx<'a> {
    ret: TypeID,
    breakable: bool,
    i: &'a UInstrInst,
}

pub fn resolve_instr<'a>(data: &mut ResData<'a>, ctx: ResolveCtx<'a>) -> Option<()> {
    let mut res = InstrRes::Finished;
    match &ctx.i.i {
        UInstruction::Call { dst, f, args } => {
            let fty = data.res_id(f, ctx)?;
            let Type::FnRef(ftyy) = data.types[fty].clone() else {
                let origin = f.origin(data);
                data.errs.push(ResErr::NotCallable { origin, ty: fty });
                return None;
            };
            let f = &data.s.fns[ftyy.id];
            for (src, dest) in args.iter().zip(&f.args) {
                res |= data.match_types(dest, src, src);
            }
            res |= data.match_types(dst, f.ret, dst);
        }
        UInstruction::Mv { dst, src } => {
            res |= data.match_types(dst, src, src);
        }
        UInstruction::Ref { dst, src } => {
            let dstid = data.res_id(dst, ctx)?;
            let Type::Ref(dest_ty) = data.types[dstid] else {
                compiler_error()
            };
            res |= data.match_types(dest_ty, src, src);
        }
        UInstruction::Deref { dst, src } => {
            let srcid = data.res_id(src, ctx)?;
            let Type::Ref(src_ty) = data.types[srcid] else {
                let origin = src.origin(data);
                data.errs.push(ResErr::CannotDeref { origin, ty: srcid });
                return None;
            };
            res |= data.match_types(dst, src_ty, src);
        }
        UInstruction::LoadData { dst, src } => {
            res |= data.match_types(dst, src, dst);
        }
        UInstruction::LoadSlice { dst, src } => {
            let dstid = data.res_id(src, ctx)?;
            let srcid = data.res_id(src, ctx)?;
            let Type::Slice(dstty) = data.types[dstid] else {
                compiler_error()
            };
            let Type::Array(srcty, _) = data.types[srcid] else {
                compiler_error()
            };
            res |= data.match_types(dstty, srcty, dst);
        }
        UInstruction::LoadFn { dst, src } => {
            // TODO: validate size with enabled targets
        }
        UInstruction::AsmBlock { instructions, args } => {
            // TODO
        }
        UInstruction::Ret { src } => {
            res |= data.match_types(ctx.ret, src, src);
        }
        UInstruction::Construct { dst, struc, fields } => {
            let id = data.res_id(dst, ctx, KindTy::Struct)?;
            let Type::Struct(sty) = &data.types[id] else {
                return None;
            };
            let sid = sty.id;
            let st = &data.s.structs[sid];
            let mut used = HashSet::new();
            for (name, field) in &st.fields {
                if let Some(src) = fields.get(name) {
                    used.insert(name);
                    res |= data.match_types(field.ty, src, src);
                } else {
                    let origin = dst.origin(data);
                    data.errs.push(ResErr::MissingField {
                        origin,
                        id: sid,
                        name: name.clone(),
                    });
                }
            }
            for (name, _) in fields {
                if !used.contains(name) {
                    let origin = dst.origin(data);
                    data.errs.push(ResErr::UnknownField {
                        origin,
                        id: sid,
                        name: name.clone(),
                    });
                }
            }
        }
        UInstruction::If { cond, body } => {
            if let Some(id) = data.res_id(cond, ctx, KindTy::Var) {
                if !matches!(data.types[id], Type::Bits(64)) {
                    let origin = cond.origin(data);
                    data.errs.push(ResErr::CondType { origin, ty: id });
                }
            }
            for i in body {
                resolve_instr(
                    data,
                    ResolveCtx {
                        ret: ctx.ret,
                        breakable: ctx.breakable,
                        i,
                    },
                );
            }
        }
        UInstruction::Loop { body } => {
            for i in body {
                resolve_instr(
                    data,
                    ResolveCtx {
                        ret: ctx.ret,
                        breakable: true,
                        i,
                    },
                );
            }
        }
        UInstruction::Break => {
            if !ctx.breakable {
                data.errs.push(ResErr::BadControlFlow {
                    op: ControlFlowOp::Break,
                    origin: ctx.i.origin,
                });
            }
        }
        UInstruction::Continue => {
            if !ctx.breakable {
                data.errs.push(ResErr::BadControlFlow {
                    op: ControlFlowOp::Continue,
                    origin: ctx.i.origin,
                });
            }
        }
    }
    match res {
        InstrRes::Finished => (),
        InstrRes::Unfinished => data.unfinished.push(ctx),
    }
    return None;
}

fn compiler_error() -> ! {
    // TODO: this is probably a compiler error / should never happen
    panic!("how could this happen to me (you)");
}

pub fn match_types(data: &mut TypeResData, dst: impl TypeIDed, src: impl TypeIDed) -> MatchRes {
    let dst = data.res_id(dst);
    let src = data.res_id(src);
    if dst == src {
        return MatchRes::Finished;
    }
    let error = || MatchRes::Error(vec![TypeMismatch { dst, src }]);
    match (data.types[dst].clone(), data.types[src].clone()) {
        (Type::Error, _) | (_, Type::Error) => MatchRes::Finished,
        (Type::Infer, Type::Infer) => MatchRes::Unfinished,
        (Type::Infer, x) => {
            *data.changed = true;
            data.types[dst] = x;
            MatchRes::Finished
        }
        (x, Type::Infer) => {
            *data.changed = true;
            data.types[src] = x;
            MatchRes::Finished
        }
        (Type::Struct(dest), Type::Struct(src)) => {
            if dest.id != src.id {
                return error();
            }
            match_all(data, dest.args.iter().cloned(), src.args.iter().cloned())
        }
        // (
        //     Type::Fn {
        //         args: dst_args,
        //         ret: dst_ret,
        //     },
        //     Type::Fn {
        //         args: src_args,
        //         ret: src_ret,
        //     },
        // ) => {
        //     let dst = dst_args.into_iter().chain(once(dst_ret));
        //     let src = src_args.into_iter().chain(once(src_ret));
        //     match_all(data, dst, src)
        // }
        (Type::Ref(dest), Type::Ref(src)) => match_types(data, dest, src),
        (Type::Slice(dest), Type::Slice(src)) => match_types(data, dest, src),
        (Type::Array(dest, dlen), Type::Array(src, slen)) => {
            if dlen == slen {
                match_types(data, dest, src)
            } else {
                error()
            }
        }
        _ => error(),
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

fn match_all(
    data: &mut TypeResData,
    dst: impl Iterator<Item = TypeID>,
    src: impl Iterator<Item = TypeID>,
) -> MatchRes {
    let mut finished = true;
    let mut errors = Vec::new();
    for (dst, src) in dst.zip(src) {
        match match_types(data, dst, src) {
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

struct Sources<'a> {
    insts: &'a mut [VarInst],
    vars: &'a mut Vec<UVar>,
    fns: &'a [UFunc],
    structs: &'a [UStruct],
    generics: &'a [UGeneric],
    data: &'a [UData],
    modules: &'a [UModule],
}

struct ResData<'a> {
    unfinished: Vec<ResolveCtx<'a>>,
    changed: bool,
    types: &'a mut Vec<Type>,
    s: Sources<'a>,
    errs: Vec<ResErr>,
}

struct TypeResData<'a> {
    changed: &'a mut bool,
    types: &'a mut [Type],
    sources: &'a Sources<'a>,
}

impl<'a> ResData<'a> {
    pub fn match_types(
        &mut self,
        dst: impl ResID<Type>,
        src: impl ResID<Type>,
        origin: impl HasOrigin,
    ) -> InstrRes {
        let dst = dst.try_id(&mut self.s, self.types, &mut self.errs, KindTy::Type)?;
        let src = src.try_id(&mut self.s, self.types, &mut self.errs, KindTy::Type)?;
        let res = match_types(
            &mut TypeResData {
                changed: &mut self.changed,
                types: self.types,
                sources: &self.s,
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
                    origin: origin.origin(self),
                    dst,
                    src,
                });
                InstrRes::Finished
            }
        }
    }
    pub fn try_res_id<K>(&mut self, x: impl ResID) -> Result<ID<K>, InstrRes> {
        x.try_id(&mut self.s, &mut self.types, &mut self.errs)
            .map(|id| resolve_refs(self.types, id))
    }
    pub fn res_id<'b: 'a, K>(
        &mut self,
        x: impl ResID<K>,
        ctx: ResolveCtx<'b>,
    ) -> Option<ID<K>> {
        match self.try_res_id(x) {
            Ok(id) => return Some(id),
            Err(InstrRes::Unfinished) => self.unfinished.push(ctx),
            Err(InstrRes::Finished) => (),
        }
        None
    }
}

impl TypeResData<'_> {
    pub fn res_id(&self, x: impl TypeIDed) -> TypeID {
        resolve_refs(self.types, x.type_id(self.sources))
    }
}

pub enum MatchRes {
    Unfinished,
    Finished,
    Error(Vec<TypeMismatch>),
}

#[derive(Debug, Clone, Copy)]
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

impl FromResidual<Option<Infallible>> for InstrRes {
    fn from_residual(_: Option<Infallible>) -> Self {
        Self::Unfinished
    }
}

trait ResID<K> {
    fn try_id(
        &self,
        s: &mut Sources,
        types: &mut Vec<Type>,
        errs: &mut Vec<ResErr>,
    ) -> Result<ID<K>, InstrRes>;
}

impl<T: TypeIDed> ResID<Type> for T {
    fn try_id(
        &self,
        s: &mut Sources,
        _: &mut Vec<Type>,
        errs: &mut Vec<ResErr>,
        kind: KindTy,
    ) -> Result<TypeID, InstrRes> {
        Ok(self.type_id(s))
    }
}

impl VarInst {
    pub fn resolve(&mut self, s: &mut Sources) {
        match &self.status {
            VarStatus::Var(id) => self.status = VarStatus::Cooked,
            VarStatus::Struct(id, ids) => todo!(),
            VarStatus::Unres { path, name, gargs, fields } => todo!(),
            VarStatus::Partial { v, fields } => todo!(),
            VarStatus::Cooked => todo!(),
        }
    }
}

impl<K: Kind> ResID<K> for VarInstID {
    fn try_id(
        &self,
        s: &mut Sources,
        types: &mut Vec<Type>,
        errs: &mut Vec<ResErr>,
    ) -> Result<ID<K>, InstrRes> {
        let kind = K::ty();
        let inst = &mut s.insts[self];
        let (id, fields) = match &mut inst.status {
            VarStatus::Var(id) => {
                return Ok(s.vars[id].ty)
            },
            VarStatus::Unres {
                path,
                name,
                gargs,
                fields,
            } => {
                let mut mid = path.id;
                let mut depth = 0;
                for mem in &path.path {
                    let Some(&child) = s.modules[mid].children.get(&mem.name) else {
                        break;
                    };
                    depth += 1;
                    mid = child;
                }
                path.path.drain(0..depth);
                path.id = mid;
                if path.path.len() != 0 {
                    return Err(InstrRes::Unfinished);
                }
                let Some(mem) = s.modules[mid].members.get(name) else {
                    return Err(InstrRes::Unfinished);
                };
                let vid = match mem.id {
                    MemberTy::Fn(id) => {
                        if kind == KindTy::Fn {
                            return Ok(id.0.into());
                        }
                        if !matches!(kind, KindTy::Var | KindTy::Fn) {
                            errs.push(ResErr::KindMismatch {
                                origin: inst.origin,
                                expected: kind,
                                found: KindTy::Fn,
                                id: id.0,
                            });
                            return Err(InstrRes::Finished);
                        }
                        inst_fn_var(
                            id,
                            s.fns,
                            gargs,
                            inst.origin,
                            s.vars,
                            types,
                            s.generics,
                            errs,
                        )
                    }
                    MemberTy::Var(id) => {
                        if !matches!(kind, KindTy::Var | KindTy::Any) {
                            errs.push(ResErr::KindMismatch {
                                origin: inst.origin,
                                expected: kind,
                                found: KindTy::Var,
                                id: id.0,
                            });
                            return Err(InstrRes::Finished);
                        }
                        if !gargs.is_empty() {
                            errs.push(ResErr::GenericCount {
                                origin: inst.origin,
                                expected: 0,
                                found: gargs.len(),
                            });
                        }
                        id
                    }
                    MemberTy::Struct(id) => {
                        if !matches!(kind, KindTy::Struct | KindTy::Type | KindTy::Any) {
                            errs.push(ResErr::KindMismatch {
                                origin: inst.origin,
                                expected: kind,
                                found: KindTy::Struct,
                                id: id.0,
                            });
                            return Err(InstrRes::Finished);
                        }
                        if fields.len() > 0 {
                            errs.push(ResErr::UnexpectedField {
                                origin: inst.origin,
                            });
                            return Err(InstrRes::Finished);
                        }
                        return Ok(inst_struct_ty(
                            id, s.structs, gargs, types, s.generics, errs,
                        ));
                    }
                };
                if fields.len() > 0 {
                    inst.status = VarStatus::Partial{v: vid, fields}
                }
            }
            VarStatus::Partial { v, fields } => (*v, fields),
            VarStatus::Cooked => return Err(InstrRes::Finished),
        };
        // I feel like this clone is not necessary but idk how
        inst.status = VarStatus::Partial {
            v: id,
            fields: fields.clone(),
        };
        // let VarStatus::Partial { v, fields } = inst.status
        todo!()
    }
}

impl<K> ResID<K> for &VarInstID {
    fn try_id(
        &self,
        s: &mut Sources,
        types: &mut Vec<Type>,
        errs: &mut Vec<ResErr>,
        kind: KindTy,
    ) -> Result<ID<K>, InstrRes> {
        (*self).try_id(s, types, errs, kind)
    }
}

pub trait TypeIDed {
    fn type_id(&self, s: &Sources) -> TypeID;
}

impl TypeIDed for TypeID {
    fn type_id(&self, _: &Sources) -> TypeID {
        *self
    }
}

impl TypeIDed for VarID {
    fn type_id(&self, s: &Sources) -> TypeID {
        s.vars[self].ty
    }
}

impl TypeIDed for DataID {
    fn type_id(&self, s: &Sources) -> TypeID {
        s.data[self].ty
    }
}

impl<T: TypeIDed> TypeIDed for &T {
    fn type_id(&self, s: &Sources) -> TypeID {
        (*self).type_id(s)
    }
}

impl FromResidual<std::result::Result<std::convert::Infallible, InstrRes>> for InstrRes {
    fn from_residual(residual: std::result::Result<std::convert::Infallible, InstrRes>) -> Self {
        match residual {
            Ok(_) => unreachable!(),
            Err(r) => r,
        }
    }
}

trait HasOrigin {
    fn origin(&self, data: &ResData) -> Origin;
}

impl HasOrigin for &VarInstID {
    fn origin(&self, data: &ResData) -> Origin {
        data.s.insts[*self].origin
    }
}
