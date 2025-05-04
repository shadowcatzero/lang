use super::{
    inst_fn_ty, inst_struct_ty, report_errs, ControlFlowOp, DataID, FnInst, IdentID, IdentStatus,
    MemberTy, Origin, ResErr, StructInst, Type, TypeID, TypeMismatch, UData, UFunc, UGeneric,
    UIdent, UInstrInst, UInstruction, UModule, UProgram, UStruct, UVar, VarID,
};
use crate::{
    common::{CompilerMsg, CompilerOutput},
    ir::{inst_fn_var, ID},
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
        let mut errs = data.errs;
        for ident in &self.idents {
            match &ident.status {
                IdentStatus::Unres {
                    path,
                    mem,
                    gargs,
                    fields,
                } => errs.push(ResErr::UnknownModule {
                    origin: path.path[0].origin,
                    name: path.path[0].name.clone(),
                }),
                IdentStatus::PartialVar { id, fields } => todo!(),
                IdentStatus::Failed(err) => errs.push(err.clone()),
                _ => (),
            }
        }
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
            let fi = data.res_id::<UFunc>(f, ctx)?;
            let f = &data.s.fns[fi.id];
            for (src, dest) in args.iter().zip(&f.args) {
                res |= data.match_types(dest, src, src);
            }
            res |= data.match_types(dst, f.ret, dst);
        }
        UInstruction::Mv { dst, src } => {
            res |= data.match_types(dst, src, src);
        }
        UInstruction::Ref { dst, src } => {
            let dstid = data.res_ty_id::<UVar>(dst, ctx)?;
            let Type::Ref(dest_ty) = data.types[dstid] else {
                compiler_error()
            };
            res |= data.match_types(dest_ty, src, src);
        }
        UInstruction::Deref { dst, src } => {
            let srcid = data.res_ty_id::<UVar>(src, ctx)?;
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
            let si = data.res_id::<UStruct>(dst, ctx)?;
            let sid = si.id;
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
            if let Some(id) = data.res_ty_id::<UVar>(cond, ctx) {
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

struct TypeResData<'a> {
    changed: &'a mut bool,
    types: &'a mut [Type],
    sources: &'a Sources<'a>,
}

impl<'a> ResData<'a> {
    pub fn match_types(
        &mut self,
        dst: impl Resolvable<Type>,
        src: impl Resolvable<Type>,
        origin: impl HasOrigin,
    ) -> InstrRes {
        let dst = dst.try_res(&mut self.s, self.types, &mut self.errs)?;
        let src = src.try_res(&mut self.s, self.types, &mut self.errs)?;
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
    pub fn try_res_id<K: ResKind>(&mut self, x: impl Resolvable<K>) -> Result<K::Res, InstrRes> {
        x.try_res(&mut self.s, &mut self.types, &mut self.errs)
    }
    pub fn res_ty_id<'b: 'a, K: ResKind>(
        &mut self,
        x: impl Resolvable<K>,
        ctx: ResolveCtx<'b>,
    ) -> Option<TypeID>
    where
        K::Res: TypeIDed,
    {
        self.res_id::<K>(x, ctx).map(|i| i.type_id(&self.s))
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

trait Resolvable<K: ResKind> {
    fn try_res(
        &self,
        s: &mut Sources,
        types: &mut Vec<Type>,
        errs: &mut Vec<ResErr>,
    ) -> Result<K::Res, InstrRes>;
}

impl<T: TypeIDed> Resolvable<Type> for T {
    fn try_res(
        &self,
        s: &mut Sources,
        _: &mut Vec<Type>,
        errs: &mut Vec<ResErr>,
    ) -> Result<TypeID, InstrRes> {
        Ok(self.type_id(s))
    }
}

impl IdentID {
    pub fn resolve(self, s: &mut Sources) -> Result<Res, InstrRes> {
        let ident = &mut s.idents[self];
        Ok(match &mut ident.status {
            IdentStatus::Var(id) => Res::Var(*id),
            IdentStatus::Struct(sty) => Res::Struct(sty.clone()),
            IdentStatus::Fn(fty) => Res::Fn(fty.clone()),
            IdentStatus::Type(ty) => Res::Type(*ty),
            IdentStatus::Unres {
                path,
                mem,
                gargs,
                fields,
            } => {
                let mut mid = path.id;
                let mut count = 0;
                for mem in &path.path {
                    let Some(&child) = s.modules[mid].children.get(&mem.name) else {
                        break;
                    };
                    count += 1;
                    mid = child;
                }
                path.path.drain(0..count);
                path.id = mid;
                if path.path.len() != 0 {
                    return Err(InstrRes::Unfinished);
                }
                let Some(mem) = s.modules[mid].members.get(&mem.name) else {
                    return Err(InstrRes::Unfinished);
                };
                match mem.id {
                    MemberTy::Fn(id) => {
                        if fields.len() > 0 {
                            ident.status = IdentStatus::Failed(ResErr::UnexpectedField {
                                origin: ident.origin,
                            });
                            return Err(InstrRes::Finished);
                        }
                        let fty = FnInst {
                            id,
                            gargs: gargs.clone(),
                        };
                        ident.status = IdentStatus::Fn(fty.clone());
                        Res::Fn(fty)
                    }
                    MemberTy::Struct(id) => {
                        if fields.len() > 0 {
                            ident.status = IdentStatus::Failed(ResErr::UnexpectedField {
                                origin: ident.origin,
                            });
                            return Err(InstrRes::Finished);
                        }
                        let sty = StructInst {
                            id,
                            gargs: gargs.clone(),
                        };
                        ident.status = IdentStatus::Struct(sty.clone());
                        Res::Struct(sty)
                    }
                    MemberTy::Var(id) => {
                        if !gargs.is_empty() {
                            ident.status = IdentStatus::Failed(ResErr::GenericCount {
                                origin: ident.origin,
                                expected: 0,
                                found: gargs.len(),
                            });
                            return Err(InstrRes::Finished);
                        }
                        ident.status = IdentStatus::PartialVar {
                            id,
                            fields: fields.clone(),
                        };
                        return self.resolve(s);
                    }
                }
            }
            IdentStatus::PartialVar { id, fields } => {
                let mut fiter = fields.iter();
                let mut next = fiter.next();
                let mut count = 0;
                while let Some(mem) = next
                    && let Some(&cid) = s.vars[*id].children.get(&mem.name)
                {
                    *id = cid;
                    next = fiter.next();
                    count += 1;
                }
                fields.drain(0..count);
                if fields.len() != 0 {
                    return Err(InstrRes::Unfinished);
                }
                let id = *id;
                ident.status = IdentStatus::Var(id);
                Res::Var(id)
            }
            IdentStatus::Cooked => return Err(InstrRes::Finished),
            IdentStatus::Failed(_) => return Err(InstrRes::Finished),
        })
    }
}

impl<K: ResKind> Resolvable<K> for IdentID {
    fn try_res(
        &self,
        s: &mut Sources,
        types: &mut Vec<Type>,
        errs: &mut Vec<ResErr>,
    ) -> Result<K::Res, InstrRes> {
        let origin = s.idents[self].origin;
        let res = self.resolve(s)?;
        match K::from_res(res.clone(), types, s, origin, errs) {
            Some(res) => Ok(res),
            None => {
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

// "effective" (externally visible) kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KindTy {
    Type,
    Var,
    Struct,
    Fn,
}

impl KindTy {
    pub fn str(&self) -> &'static str {
        match self {
            KindTy::Type => "type",
            KindTy::Var => "variable",
            KindTy::Fn => "function",
            KindTy::Struct => "struct",
        }
    }
}

#[derive(Debug, Clone)]
pub enum Res {
    Var(VarID),
    Fn(FnInst),
    Struct(StructInst),
    Type(TypeID),
}

impl Res {
    pub fn kind(&self) -> KindTy {
        match self {
            Res::Var(..) => KindTy::Var,
            Res::Fn(..) => KindTy::Fn,
            Res::Struct(..) => KindTy::Struct,
            Res::Type(..) => KindTy::Type,
        }
    }
    pub fn kind_str(&self) -> &'static str {
        self.kind().str()
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
        errs: &mut Vec<ResErr>,
    ) -> Option<Self::Res>;
}

impl ResKind for UFunc {
    type Res = FnInst;
    fn ty() -> KindTy {
        KindTy::Fn
    }
    fn from_res(
        res: Res,
        _: &mut Vec<Type>,
        _: &mut Sources,
        _: Origin,
        _: &mut Vec<ResErr>,
    ) -> Option<Self::Res> {
        match res {
            Res::Fn(fi) => Some(fi),
            _ => None,
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
        errs: &mut Vec<ResErr>,
    ) -> Option<Self::Res> {
        Some(match res {
            Res::Fn(fty) => inst_fn_var(&fty, s.fns, origin, s.vars, types, s.generics, errs),
            Res::Var(id) => id,
            Res::Struct(_) => return None,
            Res::Type(_) => return None,
        })
    }
}

impl ResKind for UStruct {
    type Res = StructInst;
    fn ty() -> KindTy {
        KindTy::Struct
    }
    fn from_res(
        res: Res,
        _: &mut Vec<Type>,
        _: &mut Sources,
        _: Origin,
        _: &mut Vec<ResErr>,
    ) -> Option<Self::Res> {
        match res {
            Res::Struct(si) => Some(si),
            _ => None,
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
        errs: &mut Vec<ResErr>,
    ) -> Option<Self::Res> {
        Some(match res {
            Res::Fn(fty) => inst_fn_ty(&fty, s.fns, types, s.generics, errs),
            Res::Var(id) => id.type_id(s),
            Res::Struct(si) => inst_struct_ty(&si, s.structs, types, s.generics, errs),
            Res::Type(id) => id,
        })
    }
}

impl<K: ResKind> Resolvable<K> for &IdentID {
    fn try_res(
        &self,
        s: &mut Sources,
        types: &mut Vec<Type>,
        errs: &mut Vec<ResErr>,
    ) -> Result<K::Res, InstrRes> {
        Resolvable::<K>::try_res(*self, s, types, errs)
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

impl HasOrigin for &IdentID {
    fn origin(&self, data: &ResData) -> Origin {
        data.s.idents[*self].origin
    }
}
