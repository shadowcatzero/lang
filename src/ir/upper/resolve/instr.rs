use std::collections::HashSet;

use super::*;

pub enum UResEvent {
    VarUse(VarID),
}

impl UProgram {
    pub fn resolve_instrs(&mut self, errs: &mut Vec<ResErr>) -> ResolveRes {
        let mut data = ResData {
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
            errs,
        };
        for ids in std::mem::take(&mut self.unres_instrs) {
            if let ResolveRes::Unfinished = resolve_instr(ids, &mut self.instrs, &mut data) {
                self.unres_instrs.push(ids);
            };
        }
        ResolveRes::Finished
    }
}

#[derive(Clone, Copy)]
struct ResolveCtx {
    ret: IdentID,
    breakable: bool,
    i: InstrID,
}

pub fn resolve_instr<'a>(
    (fi, ii): (FnID, InstrID),
    instrs: &mut Vec<UInstrInst>,
    data: &mut ResData<'a>,
) -> ResolveRes {
    let instr = &mut instrs[ii];
    match &mut instr.i {
        UInstruction::Call { dst, f, args } => {
            let fi = data.res::<UFunc>(*f);
            for &a in args {
                data.res::<UVar>(a);
            }
            data.res::<UVar>(dst);
            match fi {
                Ok(fi) => {
                    let f = &data.s.fns[fi.id];
                    for (&src, &dst) in args.iter().zip(&f.args) {
                        data.s.constraints.push(UResEvent::AssignVVI { dst, src });
                    }
                }
                Err(r) => return r,
            }
            ResolveRes::Finished
        }
        UInstruction::Mv { dst, src } => {
            res |= data.match_types::<UVar, UVar>(dst, src, src);
        }
        UInstruction::Ref { dst, src } => {
            let dstty = &data.types[data.res_var_ty(dst)?];
            let &Type::Ref(dest_ty) = dstty else {
                compiler_error()
            };
            res |= data.match_types::<Type, UVar>(dest_ty, src, src);
        }
        UInstruction::Deref { dst, src } => {
            let srcid = data.res_var_ty(src)?;
            let &Type::Ref(src_ty) = data.types[srcid] else {
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
            let (dstty, dstid) = data.res_var_ty(dst, ctx)?;
            let &Type::Slice(dstty) = dstty else {
                compiler_error()
            };
            let srcid = src.type_id(&data.s);
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
            let si = data.res::<UStruct>(dst, ctx)?;
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
            if let Some(ty) = data.res_var_ty(cond, ctx) {
                if !matches!(ty.0, RType::Bits(64)) {
                    let id = ty.1;
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
}
