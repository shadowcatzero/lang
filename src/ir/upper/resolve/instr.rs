use std::collections::HashSet;

use super::*;

pub fn resolve_instr<'a>(data: &mut ResData<'a>, ctx: ResolveCtx<'a>) -> Option<()> {
    let mut res = ResolveRes::Finished;
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
            let dstty = data.res_var_ty(dst, ctx)?.0;
            let &RType::Ref(dest_ty) = dstty else {
                compiler_error()
            };
            res |= data.match_types::<Type, UVar>(dest_ty, src, src);
        }
        UInstruction::Deref { dst, src } => {
            let (srcty, srcid) = data.res_var_ty(src, ctx)?;
            let &RType::Ref(src_ty) = srcty else {
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
            let &RType::Slice(dstty) = dstty else {
                compiler_error()
            };
            let srcid = src.type_id(&data.s);
            let Type::Real(RType::Array(srcty, _)) = data.types[srcid] else {
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
    match res {
        ResolveRes::Finished => (),
        ResolveRes::Unfinished => data.unfinished.push(ctx),
    }
    return None;
}
