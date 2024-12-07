use super::{func::FnLowerCtx, FnLowerable, PExpr, UnaryOp};
use crate::ir::{IRUInstruction, Type, VarID};

impl FnLowerable for PExpr {
    type Output = VarID;
    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<VarID> {
        Some(match self {
            PExpr::Lit(l) => match l.as_ref()? {
                super::PLiteral::String(s) => {
                    let dest = ctx.map.temp_var(l.span, Type::Bits(8).arr().rf());
                    let src = ctx.map.def_data(s.as_bytes().to_vec());
                    ctx.push(IRUInstruction::LoadData { dest, src });
                    dest
                }
                super::PLiteral::Char(c) => {
                    let dest = ctx.map.temp_var(l.span, Type::Bits(8).arr().rf());
                    let src = ctx.map.def_data(c.to_string().as_bytes().to_vec());
                    ctx.push(IRUInstruction::LoadData { dest, src });
                    dest
                }
                super::PLiteral::Number(n) => {
                    // TODO: temp
                    let dest = ctx.map.temp_var(l.span, Type::Bits(8).arr().rf());
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
                let f = e.lower(ctx)?;
                let mut nargs = Vec::new();
                for arg in args.iter() {
                    let arg = arg.lower(ctx)?;
                    nargs.push(arg);
                }
                let temp = ctx.temp(ctx.map.get_fn_var(f).ret.clone());
                ctx.push(IRUInstruction::Call {
                    dest: temp,
                    f,
                    args: nargs,
                });
                // ctx.err(format!("Expected function, found {:?}", f));
                return None;
            }
            PExpr::Group(e) => e.lower(ctx)?,
        })
    }
}
