use super::{func::FnLowerCtx, FnLowerable, PExpr, UnaryOp};
use crate::ir::{IRUInstruction, Size, Type, VarID};

impl FnLowerable for PExpr {
    type Output = VarID;
    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<VarID> {
        Some(match self {
            PExpr::Lit(l) => match l.as_ref()? {
                super::PLiteral::String(s) => {
                    let dest = ctx.map.temp_var(l.span, Type::Bits(8).slice());
                    let data = s.as_bytes().to_vec();
                    let len = data.len() as Size;
                    let src = ctx.map.def_data(data);
                    ctx.push(IRUInstruction::LoadSlice { dest, src, len });
                    dest
                }
                super::PLiteral::Char(c) => {
                    let dest = ctx.map.temp_var(l.span, Type::Bits(8).slice());
                    let src = ctx.map.def_data(c.to_string().as_bytes().to_vec());
                    ctx.push(IRUInstruction::LoadData { dest, src });
                    dest
                }
                super::PLiteral::Number(n) => {
                    // TODO: temp
                    let dest = ctx.map.temp_var(l.span, Type::Bits(64));
                    let src = ctx
                        .map
                        .def_data(n.whole.parse::<i64>().unwrap().to_le_bytes().to_vec());
                    ctx.push(IRUInstruction::LoadData { dest, src });
                    dest
                }
                super::PLiteral::Unit => {
                    todo!();
                }
            },
            PExpr::Ident(i) => ctx.get_var(i)?,
            PExpr::BinaryOp(op, e1, e2) => {
                let res1 = e1.lower(ctx)?;
                let res2 = e2.lower(ctx)?;
                op.traitt();
                todo!();
            }
            PExpr::UnaryOp(op, e) => {
                let res = e.lower(ctx)?;
                match op {
                    UnaryOp::Ref => {
                        let temp = ctx.temp(ctx.map.get_var(res).ty.clone());
                        ctx.push(IRUInstruction::Ref {
                            dest: temp,
                            src: res,
                        });
                        temp
                    }
                    UnaryOp::Deref => {
                        let t = &ctx.map.get_var(res).ty;
                        let Type::Ref(inner) = t else {
                            ctx.err(format!(
                                "Cannot dereference type {:?}",
                                ctx.map.type_name(t)
                            ));
                            return None;
                        };
                        todo!();
                    }
                    UnaryOp::Not => todo!(),
                }
            }
            PExpr::Block(b) => b.lower(ctx)?,
            PExpr::AsmBlock(b) => {
                b.lower(ctx);
                return None;
            }
            PExpr::Call(e, args) => {
                let fe = e.lower(ctx)?;
                let mut nargs = Vec::new();
                for arg in args.iter() {
                    let arg = arg.lower(ctx)?;
                    nargs.push(arg);
                }
                let def = ctx.map.get_fn_var(fe);
                let ty = match def {
                    Some(def) => def.ret.clone(),
                    None => {
                        ctx.err_at(
                            e.span,
                            format!(
                                "Expected function, found {}",
                                ctx.map.type_name(&ctx.map.get_var(fe).ty)
                            ),
                        );
                        Type::Error
                    }
                };
                let temp = ctx.temp(ty);
                ctx.push(IRUInstruction::Call {
                    dest: temp,
                    f: fe,
                    args: nargs,
                });
                temp
            }
            PExpr::Group(e) => e.lower(ctx)?,
        })
    }
}
