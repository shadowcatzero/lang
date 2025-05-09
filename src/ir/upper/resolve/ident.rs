use super::*;

impl UProgram {
    pub fn resolve_idents(&mut self, errs: &mut Vec<ResErr>) -> ResolveRes {
        let mut resolve_res = ResolveRes::Finished;
        'main: for i in std::mem::take(&mut self.unres_idents) {
            let mut j = i;
            // take from ref if possible
            while let IdentStatus::Ref(other) = &self.idents[j].status {
                match &self.idents[other].status {
                    IdentStatus::Res(res) => self.idents[i].status = IdentStatus::Res(res.clone()),
                    &IdentStatus::Ref(id) => j = id,
                    IdentStatus::Unres { .. } => {
                        self.unres_idents.push(i);
                        continue 'main;
                    }
                    IdentStatus::Failed(..) => self.idents[i].status = IdentStatus::Cooked,
                    IdentStatus::Cooked => self.idents[i].status = IdentStatus::Cooked,
                }
            }
            let status = &mut self.idents[i].status;
            // TOOD: there are some clones here that shouldn't be needed
            let IdentStatus::Unres { path, base } = status else {
                continue;
            };

            while let Some(mem) = path.pop() {
                let res = match base {
                    ResBase::Unvalidated(u) => {
                        match u.validate(
                            &self.fns,
                            &self.structs,
                            &self.generics,
                            &mut self.types,
                            errs,
                        ) {
                            Ok(res) => res,
                            Err(err) => {
                                *status = IdentStatus::Failed(err);
                                continue 'main;
                            }
                        }
                    }
                    ResBase::Validated(res) => res.clone(),
                };
                *base = match (res, mem.ty) {
                    (Res::Module(id), MemberTy::Member) => {
                        let Some(m) = self.modules[id].members.get(&mem.name) else {
                            self.unres_idents.push(i);
                            continue 'main;
                        };
                        ResBase::Unvalidated(MemRes {
                            mem: m.clone(),
                            origin: mem.origin,
                            gargs: mem.gargs,
                        })
                    }
                    (Res::Var(id), MemberTy::Field) => {
                        // trait resolution here
                        let Some(&child) = self.vars[id].children.get(&mem.name) else {
                            self.unres_idents.push(i);
                            continue 'main;
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
                        continue 'main;
                    }
                };
            }
            let res = match base {
                ResBase::Unvalidated(u) => {
                    match u.validate(
                        &self.fns,
                        &self.structs,
                        &self.generics,
                        &mut self.types,
                        errs,
                    ) {
                        Ok(res) => res,
                        Err(err) => {
                            *status = IdentStatus::Failed(err);
                            continue 'main;
                        }
                    }
                }
                ResBase::Validated(res) => res.clone(),
            };
            *status = IdentStatus::Res(res);
            resolve_res = ResolveRes::Unfinished;
        }
        resolve_res
    }
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

pub fn validate_gargs(
    dst: &[GenericID],
    src: &[TypeID],
    generics: &[UGeneric],
    types: &[Type],
    errs: &mut Vec<ResErr>,
    origin: Origin,
) -> Result<(), Option<ResErr>> {
    if dst.len() != src.len() {
        return Err(Some(ResErr::GenericCount {
            origin,
            expected: dst.len(),
            found: src.len(),
        }));
    }
    for (dst, src) in dst.iter().zip(src.iter()) {
        let g = &generics[dst];
        let t = &types[src];
        // TODO: validate trait constraints
    }
    Ok(())
}
