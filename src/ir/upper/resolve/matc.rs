use super::*;

pub fn match_types(data: &mut ResData, dst: impl TypeIDed, src: impl TypeIDed) -> MatchRes {
    let dstid = dst.type_id(&data.s);
    let srcid = src.type_id(&data.s);
    let dstty = data.real_ty(&dst)?.clone();
    let srcty = data.real_ty(&src)?.clone();
    let error = || {
        MatchRes::Error(vec![TypeMismatch {
            dst: dstid,
            src: srcid,
        }])
    };
    match (dstty, srcty) {
        // prefer changing dst over src
        (RType::Infer, _) => {
            data.changed = true;
            data.types[dstid] = Type::Ptr(srcid);
            dst.finish(&mut data.s, data.types);
            MatchRes::Finished
        }
        (_, RType::Infer) => {
            data.changed = true;
            data.types[srcid] = Type::Ptr(dstid);
            src.finish(&mut data.s, data.types);
            MatchRes::Finished
        }
        (RType::Struct(dest), RType::Struct(src)) => {
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
        (RType::Ref(dest), RType::Ref(src)) => match_types(data, dest, src),
        (RType::Slice(dest), RType::Slice(src)) => match_types(data, dest, src),
        (RType::Array(dest, dlen), RType::Array(src, slen)) => {
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
    pub fn match_types<Dst: ResKind, Src: ResKind>(
        &mut self,
        dst: impl Resolvable<Dst>,
        src: impl Resolvable<Src>,
        origin: impl HasOrigin,
    ) -> ResolveRes
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
    pub fn real_ty(&mut self, x: &impl TypeIDed) -> Result<&RType, MatchRes> {
        real_type(self.types, x.type_id(&self.s)).map_err(|res| match res {
            ResolveRes::Finished => MatchRes::Finished,
            ResolveRes::Unfinished => MatchRes::Unfinished,
        })
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
