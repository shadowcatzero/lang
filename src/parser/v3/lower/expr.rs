use super::{func::FnLowerCtx, FnLowerable, PExpr, UnaryOp};
use crate::{
    ir::{DataDef, IRUInstruction, Type, VarInst},
    parser::PInfixOp,
};

impl FnLowerable for PExpr {
    type Output = VarInst;
    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<VarInst> {
        Some(match self {
            PExpr::Lit(l) => match l.as_ref()? {
                super::PLiteral::String(s) => {
                    let dest = ctx.program.temp_var(l.span, Type::Bits(8).slice());
                    let data = s.as_bytes().to_vec();
                    let src = ctx.program.def_data(
                        DataDef {
                            ty: Type::Bits(8).arr(data.len() as u32),
                            origin: l.span,
                            label: format!("string \"{}\"", s.replace("\n", "\\n")),
                        },
                        data,
                    );
                    ctx.push(IRUInstruction::LoadSlice { dest, src });
                    dest
                }
                super::PLiteral::Char(c) => {
                    let ty = Type::Bits(8);
                    let dest = ctx.program.temp_var(l.span, ty.clone());
                    let src = ctx.program.def_data(
                        DataDef {
                            ty,
                            origin: l.span,
                            label: format!("char '{c}'"),
                        },
                        c.to_string().as_bytes().to_vec(),
                    );
                    ctx.push(IRUInstruction::LoadData { dest, src });
                    dest
                }
                super::PLiteral::Number(n) => {
                    // TODO: temp
                    let ty = Type::Bits(64);
                    let dest = ctx.program.temp_var(l.span, Type::Bits(64));
                    let src = ctx.program.def_data(
                        DataDef {
                            ty,
                            origin: l.span,
                            label: format!("num {n:?}"),
                        },
                        n.whole.parse::<i64>().unwrap().to_le_bytes().to_vec(),
                    );
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
                if *op == PInfixOp::Access {
                    let sty = &ctx.program.get_var(res1.id).ty;
                    let Type::Concrete(tid) = sty else {
                        ctx.err(format!(
                            "Type {:?} has no fields",
                            ctx.program.type_name(sty)
                        ));
                        return None;
                    };
                    let struc = ctx.program.get_struct(*tid);
                    let Some(box PExpr::Ident(ident)) = &e2.inner else {
                        ctx.err(format!("Field accesses must be identifiers",));
                        return None;
                    };
                    let fname = &ident.as_ref()?.0;
                    let Some(field) = struc.fields.get(fname) else {
                        ctx.err(format!("Field '{fname}' not in struct"));
                        return None;
                    };
                    let temp = ctx.temp(field.ty.clone());
                    ctx.push(IRUInstruction::Access {
                        dest: temp,
                        src: res1,
                        field: fname.to_string(),
                    });
                    temp
                } else {
                    let res2 = e2.lower(ctx)?;
                    match op {
                        PInfixOp::Add => todo!(),
                        PInfixOp::Sub => todo!(),
                        PInfixOp::Mul => todo!(),
                        PInfixOp::Div => todo!(),
                        PInfixOp::LessThan => todo!(),
                        PInfixOp::GreaterThan => todo!(),
                        PInfixOp::Access => todo!(),
                        PInfixOp::Assign => {
                            ctx.push(IRUInstruction::Mv {
                                dest: res1,
                                src: res2,
                            });
                            res1
                        }
                    }
                }
            }
            PExpr::UnaryOp(op, e) => {
                let res = e.lower(ctx)?;
                match op {
                    UnaryOp::Ref => {
                        let temp = ctx.temp(ctx.program.get_var(res.id).ty.clone().rf());
                        ctx.push(IRUInstruction::Ref {
                            dest: temp,
                            src: res,
                        });
                        temp
                    }
                    UnaryOp::Deref => {
                        let t = &ctx.program.get_var(res.id).ty;
                        let Type::Ref(inner) = t else {
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
                let def = ctx.program.get_fn_var(fe.id);
                let ty = match def {
                    Some(def) => def.ret.clone(),
                    None => {
                        ctx.err_at(
                            e.span,
                            format!(
                                "Expected function, found {}",
                                ctx.program.type_name(&ctx.program.get_var(fe.id).ty)
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
            PExpr::Construct(c) => c.lower(ctx)?,
            PExpr::If(cond, body) => {
                let cond = cond.lower(ctx)?;
                ctx.program.push();
                let mut body_ctx = ctx.branch();
                body.lower(&mut body_ctx);
                let body = body_ctx.instructions;
                ctx.program.pop();
                ctx.push(IRUInstruction::If { cond, body });
                return None;
            }
            PExpr::Loop(body) => {
                ctx.program.push();
                let mut body_ctx = ctx.branch();
                body.lower(&mut body_ctx);
                let body = body_ctx.instructions;
                ctx.program.pop();
                ctx.push(IRUInstruction::Loop { body });
                return None;
            }
            PExpr::Break => {
                ctx.push(IRUInstruction::Break);
                return None;
            }
        })
    }
}
