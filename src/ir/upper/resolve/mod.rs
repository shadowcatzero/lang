use super::*;
use crate::{
    common::CompilerOutput,
    ir::{MemRes, Member},
};
use std::{
    convert::Infallible,
    ops::{BitOrAssign, FromResidual},
};

mod error;
mod instr;
mod matc;
mod ident;
mod instantiate;

pub use error::*;
use instr::*;
use instantiate::*;

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
            if data.types[f.ret] != RType::Unit.ty()
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

fn compiler_error() -> ! {
    // TODO: this is probably a compiler error / should never happen
    panic!("how could this happen to me (you)");
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
    pub fn try_res_id<K: ResKind>(&mut self, x: impl Resolvable<K>) -> Result<K::Res, ResolveRes> {
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
    ) -> Option<(&RType, TypeID)> {
        let id = self.res_id::<UVar>(x, ctx).map(|i| i.type_id(&self.s))?;
        real_type(self.types, id).ok().map(|ty| (ty, id))
    }
    pub fn res_id<'b: 'a, K: ResKind>(
        &mut self,
        x: impl Resolvable<K>,
        ctx: ResolveCtx<'b>,
    ) -> Option<K::Res> {
        match self.try_res_id(x) {
            Ok(id) => return Some(id),
            Err(ResolveRes::Unfinished) => self.unfinished.push(ctx),
            Err(ResolveRes::Finished) => (),
        }
        None
    }

}

#[derive(Debug, Clone, Copy)]
pub enum ResolveRes {
    Finished,
    Unfinished,
}

impl BitOrAssign for ResolveRes {
    fn bitor_assign(&mut self, rhs: Self) {
        match rhs {
            ResolveRes::Finished => (),
            ResolveRes::Unfinished => *self = ResolveRes::Unfinished,
        }
    }
}

impl FromResidual<Option<Infallible>> for ResolveRes {
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
    ) -> Result<K::Res, ResolveRes>;
}

impl<K: ResKind> Resolvable<K> for IdentID {
    fn try_res(
        &self,
        s: &mut Sources,
        types: &mut Vec<Type>,
        errs: &mut Vec<ResErr>,
        changed: &mut bool,
    ) -> Result<K::Res, ResolveRes> {
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
                Err(ResolveRes::Finished)
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
    ) -> Result<K::Res, ResolveRes> {
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
    ) -> Result<<UVar as ResKind>::Res, ResolveRes> {
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
    ) -> Result<<Type as ResKind>::Res, ResolveRes> {
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
            Res::Struct(si) => push_id(types, RType::Struct(si).ty()),
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

impl FromResidual<Result<Infallible, ResolveRes>> for ResolveRes {
    fn from_residual(residual: Result<Infallible, ResolveRes>) -> Self {
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
