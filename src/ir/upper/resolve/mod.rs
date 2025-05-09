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
mod ident;
mod instantiate;
mod instr;
mod matc;

pub use error::*;
use instantiate::*;

impl UProgram {
    pub fn resolve(&mut self, output: &mut CompilerOutput) {
        self.unres_instrs = (0..self.instrs.len()).map(|i| InstrID::from(i)).collect();
        let mut res = ResolveRes::Unfinished;
        let mut errs = Vec::new();
        while res == ResolveRes::Unfinished {
            res = ResolveRes::Finished;
            res |= self.resolve_idents(&mut errs);
            res |= self.resolve_instrs(&mut errs);
        }
        for (fid, f) in self.fns.iter().enumerate() {
            // this currently works bc expressions create temporary variables
            // although you can't do things like loop {return 3} (need to analyze control flow)
            if let Some(ty) = self.res_ty(f.ret)
                && self.types[ty] != Type::Unit
                && f.instructions
                    .last()
                    .is_none_or(|i| !matches!(self.instrs[i].i, UInstruction::Ret { .. }))
            {
                errs.push(ResErr::NoReturn { fid });
            }
        }
        report_errs(self, output, errs);
    }
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
    changed: bool,
    types: &'a mut Vec<Type>,
    s: Sources<'a>,
    errs: &'a mut Vec<ResErr>,
}

impl<'a> ResData<'a> {
    pub fn res<K: ResKind>(&mut self, i: IdentID) -> Result<K::Res, ResolveRes> {
        i.res_as::<K>(&mut self.s, &mut self.types)
    }

    pub fn res_ty(&mut self, x: impl Resolvable<Type>) -> Result<TypeID, ResolveRes> {
        let id = Resolvable::<Type>::try_res(&x, &mut self.s, self.types, self.errs)?;
        resolved_type(self.types, id)
    }

    pub fn res_var_ty(&mut self, i: IdentID) -> Result<TypeID, ResolveRes> {
        let id = self.res::<UVar>(i)?;
        let id = match self.s.vars[id].ty {
            VarTy::Res(t) => Ok(t),
            VarTy::Ident(i) => i.res_as::<Type>(&mut self.s, self.types),
        }?;
        resolved_type(self.types, id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
    ) -> Result<K::Res, ResolveRes>;
}

impl IdentID {
    fn res_as<K: ResKind>(
        &self,
        s: &mut Sources,
        types: &mut Vec<Type>,
    ) -> Result<K::Res, ResolveRes> {
        let origin = s.idents[self].origin;
        let res = match &s.idents[self].status {
            IdentStatus::Res(res) => res.clone(),
            IdentStatus::Ref { .. } => return Err(ResolveRes::Unfinished),
            IdentStatus::Unres { .. } => return Err(ResolveRes::Unfinished),
            IdentStatus::Failed(..) => return Err(ResolveRes::Finished),
            IdentStatus::Cooked => return Err(ResolveRes::Finished),
        };
        match K::from_res(res, types, s, origin) {
            Ok(res) => Ok(res),
            Err(res) => {
                s.idents[self].status = IdentStatus::Failed(Some(ResErr::KindMismatch {
                    origin,
                    expected: K::ty(),
                    found: res,
                }));
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
    ) -> Result<K::Res, ResolveRes> {
        Resolvable::<K>::try_res(*self, s, types, errs)
    }
}

impl Resolvable<UVar> for VarID {
    fn try_res(
        &self,
        s: &mut Sources,
        types: &mut Vec<Type>,
        errs: &mut Vec<ResErr>,
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
            Res::Struct(si) => push_id(types, Type::Struct(si)),
            Res::Type(id) => id,
            _ => return Err(res),
        })
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
