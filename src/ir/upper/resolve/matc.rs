use super::*;

pub fn match_types(data: &mut ResData, dst: TypeID, src: TypeID) -> MatchRes {
    let Some(dst) = clean_type(data.types, dst) else {
        return MatchRes::Finished;
    };
    let Some(src) = clean_type(data.types, src) else {
        return MatchRes::Finished;
    };
    // prevents this from blowing up I think:
    // let mut x, y;
    // x = y;
    // y = x;
    if dst == src {
        return MatchRes::Finished;
    }
    let error = || MatchRes::Error(vec![TypeMismatch { dst, src }]);
    match (data.types[dst].clone(), data.types[src].clone()) {
        // prefer changing dst over src
        (Type::Infer, _) => {
            data.changed = true;
            data.types[dst] = Type::Ptr(src);
            MatchRes::Finished
        }
        (_, Type::Infer) => {
            data.changed = true;
            data.types[src] = Type::Ptr(dst);
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

impl<'a> ResData<'a> {
    pub fn match_types(
        &mut self,
        dst: impl MaybeTypeID,
        src: impl MaybeTypeID,
        origin: impl HasOrigin,
    ) -> ResolveRes {
        let dst = dst.type_id(&self.s)?;
        let src = src.type_id(&self.s)?;
        let res = match_types(self, dst, src);
        match res {
            MatchRes::Unfinished => ResolveRes::Unfinished,
            MatchRes::Finished => ResolveRes::Finished,
            MatchRes::Error(es) => {
                self.errs.push(ResErr::Type {
                    errs: es,
                    origin: origin.origin(self),
                    dst,
                    src,
                });
                ResolveRes::Finished
            }
        }
    }
}

pub enum MatchRes {
    Unfinished,
    Finished,
    Error(Vec<TypeMismatch>),
}

impl FromResidual<Result<Infallible, MatchRes>> for MatchRes {
    fn from_residual(residual: Result<Infallible, MatchRes>) -> Self {
        match residual {
            Ok(_) => unreachable!(),
            Err(r) => r,
        }
    }
}

pub trait MaybeTypeID {
    fn type_id(&self, s: &Sources) -> Result<TypeID, ResolveRes>;
}

impl<T: TypeIDed> MaybeTypeID for T {
    fn type_id(&self, s: &Sources) -> Result<TypeID, ResolveRes> {
        Ok(self.type_id(s))
    }
}

impl MaybeTypeID for VarID {
    fn type_id(&self, s: &Sources) -> Result<TypeID, ResolveRes> {
        match s.vars[self].ty {
            VarTy::Ident(id) => todo!(),
            VarTy::Res(id) => Ok(id),
        }
    }
}
