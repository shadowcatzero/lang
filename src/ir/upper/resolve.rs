use super::{
    inst_fn_var, inst_type, inst_typedef, inst_var, push_id, report_errs, resolve_refs, validate_gargs, ControlFlowOp, DataID, FnInst, IdentID, IdentStatus, KindTy, MemberID, MemberTy, Origin, Res, ResBase, ResErr, StructInst, Type, TypeID, TypeMismatch, UData, UFunc, UGeneric, UIdent, UInstrInst, UInstruction, UModule, UProgram, UStruct, UVar, VarID
};
use crate::{
    common::CompilerOutput,
    ir::{MemRes, Member},
};
use std::{
    collections::HashSet,
    convert::Infallible,
    ops::{BitOrAssign, FromResidual},
};

// dawg this file is way too long
// this is the omega file tho that's super cool

impl UProgram {
    pub fn resolve(&mut self, output: &mut CompilerOutput) {
        let mut unfinished = Vec::new();
        let mut data = ResData {
            unfinished: Vec::new(),
            changed: false,
            types: &mut self.types,
            s: Sources {
                idents: &mut self.idents,
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
            if !matches!(data.types[f.ret], Type::Unit)
                && f.instructions
                    .last()
                    .is_none_or(|i| !matches!(i.i, UInstruction::Ret { .. }))
            {
                data.errs.push(ResErr::NoReturn { fid });
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
            let fi = data.res_id::<UFunc>(f, ctx)?;
            let f = &data.s.fns[fi.id];
            for (src, &dest) in args.iter().zip(&f.args) {
                res |= data.match_types::<UVar, UVar>(dest, src, src);
            }
            res |= data.match_types::<UVar, Type>(dst, f.ret, dst);
        }
        UInstruction::Mv { dst, src } => {
            res |= data.match_types::<UVar, UVar>(dst, src, src);
        }
        UInstruction::Ref { dst, src } => {
            let dstid = data.res_var_ty(dst, ctx)?;
            let Type::Ref(dest_ty) = data.types[dstid] else {
                compiler_error()
            };
            res |= data.match_types::<Type, UVar>(dest_ty, src, src);
        }
        UInstruction::Deref { dst, src } => {
            let srcid = data.res_var_ty(src, ctx)?;
            let Type::Ref(src_ty) = data.types[srcid] else {
                let origin = src.origin(data);
                data.errs.push(ResErr::CannotDeref { origin, ty: srcid });
                return None;
            };
            res |= data.match_types::<UVar, Type>(dst, src_ty, src);
        }
        UInstruction::LoadData { dst, src } => {
            let srcid = src.type_id(&data.s);
            res |= data.match_types::<UVar, Type>(dst, srcid, dst);
        }
        UInstruction::LoadSlice { dst, src } => {
            let dstid = data.res_var_ty(dst, ctx)?;
            let srcid = src.type_id(&data.s);
            let Type::Slice(dstty) = data.types[dstid] else {
                compiler_error()
            };
            let Type::Array(srcty, _) = data.types[srcid] else {
                compiler_error()
            };
            res |= data.match_types(dstty, srcty, dst);
        }
        UInstruction::AsmBlock { instructions, args } => {
            // TODO
        }
        UInstruction::Ret { src } => {
            res |= data.match_types::<Type, UVar>(ctx.ret, src, src);
        }
        UInstruction::Construct { dst, struc, fields } => {
            let si = data.res_id::<UStruct>(dst, ctx)?;
            let sid = si.id;
            let st = &data.s.structs[sid];
            let mut used = HashSet::new();
            for (name, field) in &st.fields {
                if let Some(src) = fields.get(name) {
                    used.insert(name);
                    res |= data.match_types::<Type, UVar>(field.ty, src, src);
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
                    data.errs.push(ResErr::UnknownStructField {
                        origin,
                        id: sid,
                        name: name.clone(),
                    });
                }
            }
        }
        UInstruction::If { cond, body } => {
            if let Some(id) = data.res_var_ty(cond, ctx) {
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

pub fn match_types(data: &mut ResData, dst: impl TypeIDed, src: impl TypeIDed) -> MatchRes {
    let dstid = data.res_ty(&dst)?;
    let srcid = data.res_ty(&src)?;
    if dstid == srcid {
        return MatchRes::Finished;
    }
    let error = || {
        MatchRes::Error(vec![TypeMismatch {
            dst: dstid,
            src: srcid,
        }])
    };
    match (data.types[dstid].clone(), data.types[srcid].clone()) {
        (Type::Error, _) | (_, Type::Error) => MatchRes::Finished,
        (Type::Infer, Type::Infer) => MatchRes::Unfinished,
        (Type::Infer, x) => {
            data.changed = true;
            data.types[dstid] = x;
            dst.finish(&mut data.s, data.types);
            MatchRes::Finished
        }
        (x, Type::Infer) => {
            data.changed = true;
            data.types[srcid] = x;
            src.finish(&mut data.s, data.types);
            MatchRes::Finished
        }
        (Type::Struct(dest), Type::Struct(src)) => {
            if dest.id != src.id {
                return error();
            }
            match_all(data, dest.gargs.iter().cloned(), src.gargs.iter().cloned())
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

fn match_all(
    data: &mut ResData,
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
    idents: &'a mut [UIdent],
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

impl<'a> ResData<'a> {
    pub fn match_types<Dst: ResKind, Src: ResKind>(
        &mut self,
        dst: impl Resolvable<Dst>,
        src: impl Resolvable<Src>,
        origin: impl HasOrigin,
    ) -> InstrRes
    where
        Dst::Res: TypeIDed,
        Src::Res: TypeIDed,
    {
        let dst = dst
            .try_res(&mut self.s, self.types, &mut self.errs, &mut self.changed)?
            .type_id(&self.s);
        let src = src
            .try_res(&mut self.s, self.types, &mut self.errs, &mut self.changed)?
            .type_id(&self.s);
        let res = match_types(self, dst, src);
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
    pub fn try_res_id<K: ResKind>(&mut self, x: impl Resolvable<K>) -> Result<K::Res, InstrRes> {
        x.try_res(
            &mut self.s,
            &mut self.types,
            &mut self.errs,
            &mut self.changed,
        )
    }
    pub fn res_var_ty<'b: 'a>(
        &mut self,
        x: impl Resolvable<UVar>,
        ctx: ResolveCtx<'b>,
    ) -> Option<TypeID> {
        self.res_id::<UVar>(x, ctx).map(|i| i.type_id(&self.s))
    }
    pub fn res_id<'b: 'a, K: ResKind>(
        &mut self,
        x: impl Resolvable<K>,
        ctx: ResolveCtx<'b>,
    ) -> Option<K::Res> {
        match self.try_res_id(x) {
            Ok(id) => return Some(id),
            Err(InstrRes::Unfinished) => self.unfinished.push(ctx),
            Err(InstrRes::Finished) => (),
        }
        None
    }

    pub fn res_ty(&mut self, x: &impl TypeIDed) -> Result<TypeID, InstrRes> {
        let id = resolve_refs(self.types, x.type_id(&self.s));
        Ok(if let Type::Unres(ident) = self.types[id] {
            match self.try_res_id::<Type>(ident) {
                Ok(nid) => {
                    // this does NOT feel good lmao
                    self.types[id] = self.types[nid].clone();
                    x.finish(&mut self.s, self.types);
                    nid
                }
                Err(res) => return Err(res),
            }
        } else {
            id
        })
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

impl MemRes {
    pub fn validate(
        &self,
        fns: &[UFunc],
        structs: &[UStruct],
        generics: &[UGeneric],
        types: &mut Vec<Type>,
        errs: &mut Vec<ResErr>,
    ) -> Result<Res, Option<ResErr>> {
        let no_gargs = || {
            if self.gargs.len() > 0 {
                Err(ResErr::GenericCount {
                    origin: self.origin,
                    expected: 0,
                    found: self.gargs.len(),
                })
            } else {
                Ok(())
            }
        };
        Ok(match &self.mem.id {
            &MemberID::Fn(id) => {
                validate_gargs(
                    &fns[id].gargs,
                    &self.gargs,
                    generics,
                    types,
                    errs,
                    self.origin,
                )?;
                Res::Fn(FnInst {
                    id,
                    gargs: self.gargs.clone(),
                })
            }
            &MemberID::Struct(id) => {
                validate_gargs(
                    &structs[id].gargs,
                    &self.gargs,
                    generics,
                    types,
                    errs,
                    self.origin,
                )?;
                Res::Struct(StructInst {
                    id,
                    gargs: self.gargs.clone(),
                })
            }
            &MemberID::Var(id) => {
                no_gargs()?;
                Res::Var(id)
            }
            &MemberID::Module(id) => {
                no_gargs()?;
                Res::Module(id)
            }
            MemberID::Type(def) => {
                validate_gargs(&def.gargs, &self.gargs, generics, types, errs, self.origin)?;
                inst_typedef(def, &self.gargs, types);
                Res::Type(def.ty)
            }
        })
    }
}

impl IdentID {
    pub fn resolve(
        self,
        s: &mut Sources,
        types: &mut Vec<Type>,
        changed: &mut bool,
        errs: &mut Vec<ResErr>,
    ) -> Result<Res, InstrRes> {
        let status = &mut s.idents[self].status;
        // TOOD: there are some clones here that shouldn't be needed
        Ok(match status {
            IdentStatus::Res(r) => r.clone(),
            IdentStatus::Unres { path, base } => {
                while let Some(mem) = path.pop() {
                    let res = match base {
                        ResBase::Unvalidated(u) => {
                            match u.validate(s.fns, s.structs, s.generics, types, errs) {
                                Ok(res) => res,
                                Err(err) => {
                                    *status = IdentStatus::Failed(err);
                                    return Err(InstrRes::Finished);
                                }
                            }
                        }
                        ResBase::Validated(res) => res.clone(),
                    };
                    *base = match (res, mem.ty) {
                        (Res::Module(id), MemberTy::Member) => {
                            let Some(m) = s.modules[id].members.get(&mem.name) else {
                                return Err(InstrRes::Unfinished);
                            };
                            ResBase::Unvalidated(MemRes {
                                mem: m.clone(),
                                origin: mem.origin,
                                gargs: mem.gargs,
                            })
                        }
                        (Res::Var(id), MemberTy::Field) => {
                            // trait resolution here
                            let Some(&child) = s.vars[id].children.get(&mem.name) else {
                                return Err(InstrRes::Unfinished);
                            };
                            ResBase::Unvalidated(MemRes {
                                mem: Member {
                                    id: MemberID::Var(child),
                                },
                                origin: mem.origin,
                                gargs: mem.gargs,
                            })
                        }
                        _ => {
                            *status = IdentStatus::Failed(Some(ResErr::UnknownMember {
                                origin: mem.origin,
                                ty: mem.ty,
                                name: mem.name.clone(),
                                parent: base.clone(),
                            }));
                            return Err(InstrRes::Finished);
                        }
                    };
                }
                let res = match base {
                    ResBase::Unvalidated(u) => {
                        match u.validate(s.fns, s.structs, s.generics, types, errs) {
                            Ok(res) => res,
                            Err(err) => {
                                *status = IdentStatus::Failed(err);
                                return Err(InstrRes::Finished);
                            }
                        }
                    }
                    ResBase::Validated(res) => res.clone(),
                };
                *status = IdentStatus::Res(res.clone());
                *changed = true;
                res
            }
            IdentStatus::Cooked => return Err(InstrRes::Finished),
            IdentStatus::Failed(_) => return Err(InstrRes::Finished),
        })
    }
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

trait Resolvable<K: ResKind> {
    fn try_res(
        &self,
        s: &mut Sources,
        types: &mut Vec<Type>,
        errs: &mut Vec<ResErr>,
        changed: &mut bool,
    ) -> Result<K::Res, InstrRes>;
}

impl<K: ResKind> Resolvable<K> for IdentID {
    fn try_res(
        &self,
        s: &mut Sources,
        types: &mut Vec<Type>,
        errs: &mut Vec<ResErr>,
        changed: &mut bool,
    ) -> Result<K::Res, InstrRes> {
        let origin = s.idents[self].origin;
        let res = self.resolve(s, types, changed, errs)?;
        match K::from_res(res, types, s, origin) {
            Ok(res) => Ok(res),
            Err(res) => {
                errs.push(ResErr::KindMismatch {
                    origin,
                    expected: K::ty(),
                    found: res,
                });
                Err(InstrRes::Finished)
            }
        }
    }
}

impl<K: ResKind> Resolvable<K> for &IdentID {
    fn try_res(
        &self,
        s: &mut Sources,
        types: &mut Vec<Type>,
        errs: &mut Vec<ResErr>,
        changed: &mut bool,
    ) -> Result<K::Res, InstrRes> {
        Resolvable::<K>::try_res(*self, s, types, errs, changed)
    }
}

impl Resolvable<UVar> for VarID {
    fn try_res(
        &self,
        s: &mut Sources,
        types: &mut Vec<Type>,
        errs: &mut Vec<ResErr>,
        changed: &mut bool,
    ) -> Result<<UVar as ResKind>::Res, InstrRes> {
        Ok(*self)
    }
}

impl Resolvable<Type> for TypeID {
    fn try_res(
        &self,
        s: &mut Sources,
        types: &mut Vec<Type>,
        errs: &mut Vec<ResErr>,
        changed: &mut bool,
    ) -> Result<<Type as ResKind>::Res, InstrRes> {
        Ok(*self)
    }
}

pub trait ResKind {
    type Res;
    fn ty() -> KindTy;
    fn from_res(
        res: Res,
        types: &mut Vec<Type>,
        s: &mut Sources,
        origin: Origin,
    ) -> Result<Self::Res, Res>;
}

impl ResKind for UFunc {
    type Res = FnInst;
    fn ty() -> KindTy {
        KindTy::Fn
    }
    fn from_res(res: Res, _: &mut Vec<Type>, _: &mut Sources, _: Origin) -> Result<Self::Res, Res> {
        match res {
            Res::Fn(fi) => Ok(fi),
            _ => Err(res),
        }
    }
}

impl ResKind for UVar {
    type Res = VarID;
    fn ty() -> KindTy {
        KindTy::Var
    }
    fn from_res(
        res: Res,
        types: &mut Vec<Type>,
        s: &mut Sources,
        origin: Origin,
    ) -> Result<Self::Res, Res> {
        Ok(match res {
            Res::Fn(fty) => inst_fn_var(fty, s.fns, origin, s.vars, types),
            Res::Var(id) => id,
            _ => return Err(res),
        })
    }
}

impl ResKind for UStruct {
    type Res = StructInst;
    fn ty() -> KindTy {
        KindTy::Struct
    }
    fn from_res(res: Res, _: &mut Vec<Type>, _: &mut Sources, _: Origin) -> Result<Self::Res, Res> {
        match res {
            Res::Struct(si) => Ok(si),
            _ => Err(res),
        }
    }
}

impl ResKind for Type {
    type Res = TypeID;
    fn ty() -> KindTy {
        KindTy::Type
    }
    fn from_res(
        res: Res,
        types: &mut Vec<Type>,
        s: &mut Sources,
        _: Origin,
    ) -> Result<Self::Res, Res> {
        Ok(match res {
            Res::Struct(si) => push_id(types, Type::Struct(si)),
            Res::Type(id) => id,
            _ => return Err(res),
        })
    }
}

pub trait TypeIDed {
    fn type_id(&self, s: &Sources) -> TypeID;
    fn finish(&self, s: &mut Sources, types: &mut Vec<Type>) {}
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
    fn finish(&self, s: &mut Sources, types: &mut Vec<Type>) {
        inst_var(s.vars, s.structs, *self, types);
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

impl FromResidual<Result<Infallible, InstrRes>> for InstrRes {
    fn from_residual(residual: Result<Infallible, InstrRes>) -> Self {
        match residual {
            Ok(_) => unreachable!(),
            Err(r) => r,
        }
    }
}

trait HasOrigin {
    fn origin(&self, data: &ResData) -> Origin;
}

impl HasOrigin for &IdentID {
    fn origin(&self, data: &ResData) -> Origin {
        data.s.idents[*self].origin
    }
}
impl FromResidual<Result<Infallible, MatchRes>> for MatchRes {
    fn from_residual(residual: Result<Infallible, MatchRes>) -> Self {
        match residual {
            Ok(_) => unreachable!(),
            Err(r) => r,
        }
    }
}
