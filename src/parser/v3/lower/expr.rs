use super::{func::FnLowerCtx, FnLowerable, PExpr, UnaryOp};
use crate::{
    ir::{FieldRef, Type, UData, UInstruction, VarInst},
    parser::PInfixOp,
};

impl FnLowerable for PExpr {
    type Output = VarInst;
    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<VarInst> {
        Some(match self {
            PExpr::Lit(l) => match l.as_ref()? {
                super::PLiteral::String(s) => {
                    let dest = ctx.program.temp_var(l.origin, Type::Bits(8).slice());
                    let data = s.as_bytes().to_vec();
                    let src = ctx.program.def(
                        &format!("string \"{}\"", s.replace("\n", "\\n")),
                        Some(UData {
                            ty: Type::Bits(8).arr(data.len() as u32),
                            content: data,
                        }),
                        l.origin,
                    );
                    ctx.push(UInstruction::LoadSlice { dest, src });
                    dest
                }
                super::PLiteral::Char(c) => {
                    let ty = Type::Bits(8);
                    let dest = ctx.program.temp_var(l.origin, ty.clone());
                    let src = ctx.program.def(
                        &format!("char '{c}'"),
                        Some(UData {
                            ty,
                            content: c.to_string().as_bytes().to_vec(),
                        }),
                        l.origin,
                    );
                    ctx.push(UInstruction::LoadData { dest, src });
                    dest
                }
                super::PLiteral::Number(n) => {
                    // TODO: temp
                    let ty = Type::Bits(64);
                    let dest = ctx.program.temp_var(l.origin, ty.clone());
                    let src = ctx.program.def(
                        &format!("num {n:?}"),
                        Some(UData {
                            ty,
                            content: n.whole.parse::<i64>().unwrap().to_le_bytes().to_vec(),
                        }),
                        l.origin,
                    );
                    ctx.push(UInstruction::LoadData { dest, src });
                    dest
                }
                super::PLiteral::Unit => ctx.program.temp_var(l.origin, Type::Unit),
            },
            PExpr::Ident(i) => ctx.get_var(i)?,
            PExpr::BinaryOp(op, e1, e2) => match op {
                PInfixOp::Add => todo!(),
                PInfixOp::Sub => todo!(),
                PInfixOp::Mul => todo!(),
                PInfixOp::Div => todo!(),
                PInfixOp::LessThan => todo!(),
                PInfixOp::GreaterThan => todo!(),
                PInfixOp::Member => {
                    let res1 = e1.lower(ctx)?;
                    let Some(box PExpr::Ident(ident)) = &e2.inner else {
                        ctx.err("Field accessors must be identifiers".to_string());
                        return None;
                    };
                    let fname = ident.as_ref()?.0.clone();
                    ctx.temp(Type::Field(FieldRef {
                        parent: res1.id,
                        name: fname,
                    }))
                }
                PInfixOp::Assign => {
                    let res1 = e1.lower(ctx)?;
                    let res2 = e2.lower(ctx)?;
                    ctx.push(UInstruction::Mv {
                        dest: res1,
                        src: res2,
                    });
                    res1
                }
            },
            PExpr::UnaryOp(op, e) => {
                let res = e.lower(ctx)?;
                match op {
                    UnaryOp::Ref => {
                        let temp = ctx.temp(ctx.program.expect(res.id).ty.clone().rf());
                        ctx.push(UInstruction::Ref {
                            dest: temp,
                            src: res,
                        });
                        temp
                    }
                    UnaryOp::Deref => {
                        let t = &ctx.program.expect(res.id).ty;
                        let Type::Ref(_) = t else {
                            ctx.err(format!(
                                "Cannot dereference type {:?}",
                                ctx.program.type_name(t)
                            ));
                            return None;
                        };
                        todo!();
                    }
                    UnaryOp::Not => todo!(),
                }
            }
            PExpr::Block(b) => b.lower(ctx)?,
            PExpr::AsmBlock(b) => b.lower(ctx)?,
            PExpr::Call(e, args) => {
                let fe = e.lower(ctx)?;
                let mut nargs = Vec::new();
                for arg in args.iter() {
                    let arg = arg.lower(ctx)?;
                    nargs.push(arg);
                }
                let ty = ctx
                    .program
                    .get_fn_var(fe.id)
                    .map(|f| f.ret.clone())
                    .unwrap_or(Type::Placeholder);
                let temp = ctx.temp(ty);
                ctx.push(UInstruction::Call {
                    dest: temp,
                    f: fe,
                    args: nargs,
                });
                temp
            }
            PExpr::Group(e) => e.lower(ctx)?,
            PExpr::Construct(c) => c.lower(ctx)?,
            PExpr::If(cond, body) => {
                let cond = cond.lower(ctx)?;
                ctx.program.push();
                let mut body_ctx = ctx.branch();
                body.lower(&mut body_ctx);
                let body = body_ctx.instructions;
                ctx.program.pop();
                ctx.push(UInstruction::If { cond, body });
                return None;
            }
            PExpr::Loop(body) => {
                ctx.program.push();
                let mut body_ctx = ctx.branch();
                body.lower(&mut body_ctx);
                let body = body_ctx.instructions;
                ctx.program.pop();
                ctx.push(UInstruction::Loop { body });
                return None;
            }
            PExpr::Break => {
                ctx.push(UInstruction::Break);
                return None;
            }
            PExpr::Continue => {
                ctx.push(UInstruction::Continue);
                return None;
            }
        })
    }
}
