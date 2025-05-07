use super::{func::FnLowerCtx, FnLowerable, PExpr, PostfixOp};
use crate::{
    ir::{IdentID, IdentStatus, MemRes, Member, MemberID, MemberIdent, Type, UData, UInstruction},
    parser::InfixOp,
};

impl FnLowerable for PExpr {
    type Output = IdentID;
    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<IdentID> {
        let mut e = self;
        let mut path = Vec::new();
        let mut gargs = None;
        loop {
            match e {
                PExpr::Member(node, ty, ident) => {
                    e = if let Some(t) = node.as_ref() {
                        ctx.origin = node.origin;
                        path.push((ty, ident, gargs.unwrap_or_default()));
                        &**t
                    } else {
                        return None;
                    };
                }
                PExpr::Generic(node, nodes) => match gargs {
                    None => gargs = Some(nodes.iter().map(|t| t.lower(ctx)).collect::<Vec<_>>()),
                    Some(_) => {
                        // this should cover the more specific area of ::<...>
                        // but too lazy rn
                        ctx.err("Cannot specify generics here".to_string());
                        return None;
                    }
                },
                _ => break,
            }
        }
        while let PExpr::Member(node, ty, ident) = e {}
        if path.len() > 0 {
            // UIdent {
            //     origin: ctx.origin,
            //     status: IdentStatus::Unres { base: (), path: () },
            // }
        }
        let origin = ctx.origin;
        Some(match e {
            PExpr::Lit(l) => match l {
                super::PLiteral::String(s) => {
                    let sty = Type::Bits(8).slice(ctx.p);
                    let dst = ctx.temp_var(origin, sty);
                    let data = s.as_bytes().to_vec();
                    let dty = Type::Bits(8).arr(ctx.ctx.p, data.len() as u32);
                    let dty = ctx.def_ty(dty);
                    let src = ctx.def_data(UData {
                        name: format!("string \"{}\"", s.replace("\n", "\\n")),
                        ty: dty,
                        content: data,
                    });
                    ctx.push(UInstruction::LoadSlice { dst, src });
                    dst
                }
                super::PLiteral::Char(c) => {
                    let ty = ctx.def_ty(Type::Bits(8));
                    let dst = ctx.temp_var(origin, ty.clone());
                    let src = ctx.def_data(UData {
                        name: format!("char '{c}'"),
                        ty,
                        content: c.to_string().as_bytes().to_vec(),
                    });
                    ctx.push(UInstruction::LoadData { dst, src });
                    dst
                }
                super::PLiteral::Number(n) => {
                    // TODO: temp
                    let ty = ctx.def_ty(Type::Bits(64));
                    let dst = ctx.temp_var(origin, ty.clone());
                    let src = ctx.def_data(UData {
                        name: format!("num {n:?}"),
                        ty,
                        content: n.whole.parse::<i64>().unwrap().to_le_bytes().to_vec(),
                    });
                    ctx.push(UInstruction::LoadData { dst, src });
                    dst
                }
                super::PLiteral::Unit => ctx.temp_var(origin, Type::Unit),
            },
            PExpr::Ident(i) => ctx.ident(i),
            PExpr::BinaryOp(op, e1, e2) => match op {
                InfixOp::Add => todo!(),
                InfixOp::Sub => todo!(),
                InfixOp::Mul => todo!(),
                InfixOp::Div => todo!(),
                InfixOp::LessThan => todo!(),
                InfixOp::GreaterThan => todo!(),
                InfixOp::Assign => {
                    let res1 = e1.lower(ctx)?;
                    let res2 = e2.lower(ctx)?;
                    ctx.push(UInstruction::Mv {
                        dst: res1,
                        src: res2,
                    });
                    res1
                }
            },
            PExpr::PostfixOp(e, op) => {
                let res = e.lower(ctx)?;
                match op {
                    PostfixOp::Ref => {
                        let ty = Type::Ref(ctx.ctx.infer());
                        let dest = ctx.temp(ty);
                        ctx.push(UInstruction::Ref {
                            dst: dest,
                            src: res,
                        });
                        dest
                    }
                    PostfixOp::Deref => {
                        let ty = Type::Deref(ctx.ctx.infer());
                        let dst = ctx.temp(ty);
                        ctx.push(UInstruction::Deref { dst, src: res });
                        dst
                    }
                    PostfixOp::Not => todo!(),
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
                let dest = ctx.temp(Type::Infer);
                ctx.push(UInstruction::Call {
                    dst: dest,
                    f: fe,
                    args: nargs,
                });
                dest
            }
            PExpr::Group(e) => e.lower(ctx)?,
            PExpr::Construct(e, map) => {
                let dst = ctx.temp(Type::Infer);
                let struc = e.lower(ctx)?;
                let fields = map.lower(ctx)?;
                ctx.push(UInstruction::Construct { dst, struc, fields });
                dst
            }
            PExpr::If(cond, body) => {
                let cond = cond.lower(ctx)?;
                ctx.ident_stack.push();
                let mut body_ctx = ctx.branch();
                body.lower(&mut body_ctx);
                let body = body_ctx.instructions;
                ctx.ident_stack.pop();
                ctx.push(UInstruction::If { cond, body });
                return None;
            }
            PExpr::Loop(body) => {
                ctx.ident_stack.push();
                let mut body_ctx = ctx.branch();
                body.lower(&mut body_ctx);
                let body = body_ctx.instructions;
                ctx.ident_stack.pop();
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
            PExpr::Member(e, ty, name) => {
                let id = e.lower(ctx)?;
                let name_str = name.as_ref()?.0;
                let cur = &mut ctx.p.idents[id];
                match cur.status {
                    IdentStatus::Res(res) => {
                        cur.status = IdentStatus::Unres {
                            base: MemRes {
                                mem: Member {
                                    id: MemberID
                                },
                                origin: (),
                                gargs: (),
                            },
                            path: (),
                        }
                    }
                    IdentStatus::Unres { base, path } => path.push(MemberIdent {
                        ty: *ty,
                        name: name_str,
                        origin: name.origin,
                        gargs: Vec::new(),
                    }),
                    IdentStatus::Failed(res_err) => return None,
                    IdentStatus::Cooked => return None,
                }
                return None;
            }
        })
    }
}
