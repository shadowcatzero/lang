use super::{func::FnLowerCtx, FnLowerable, PExpr, PostfixOp};
use crate::{
    ir::{Type, UData, UInstruction, VarInst},
    parser::InfixOp,
};

impl FnLowerable for PExpr {
    type Output = VarInst;
    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<VarInst> {
        let mut e = self;
        let mut path = Vec::new();
        while let PExpr::Member(node, ident) = e {
            e = if let Some(t) = node.as_ref() {
                path.push(ident);
                &**t
            } else {
                return None;
            };
        }
        Some(match e {
            PExpr::Lit(l) => match l {
                super::PLiteral::String(s) => {
                    let dest = ctx.b.temp_var(ctx.origin, Type::Bits(8).slice(ctx.b.p));
                    let data = s.as_bytes().to_vec();
                    let src = ctx.b.def_data(UData {
                        name: format!("string \"{}\"", s.replace("\n", "\\n")),
                        ty: Type::Bits(8).arr(ctx.b.p, data.len() as u32),
                        content: data,
                    });
                    ctx.push(UInstruction::LoadSlice { dst: dest, src });
                    dest
                }
                super::PLiteral::Char(c) => {
                    let ty = Type::Bits(8);
                    let dest = ctx.b.temp_var(ctx.origin, ty.clone());
                    let src = ctx.b.def_data(UData {
                        name: format!("char '{c}'"),
                        ty,
                        content: c.to_string().as_bytes().to_vec(),
                    });
                    ctx.push(UInstruction::LoadData { dst: dest, src });
                    dest
                }
                super::PLiteral::Number(n) => {
                    // TODO: temp
                    let ty = Type::Bits(64);
                    let dest = ctx.b.temp_var(ctx.origin, ty.clone());
                    let src = ctx.b.def_data(UData {
                        name: format!("num {n:?}"),
                        ty,
                        content: n.whole.parse::<i64>().unwrap().to_le_bytes().to_vec(),
                    });
                    ctx.push(UInstruction::LoadData { dst: dest, src });
                    dest
                }
                super::PLiteral::Unit => ctx.b.temp_var(ctx.origin, Type::Unit),
            },
            PExpr::Ident(i) => ctx.var(i),
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
                        let ty = Type::Ref(ctx.b.infer());
                        let dest = ctx.temp(ty);
                        ctx.push(UInstruction::Ref { dst: dest, src: res });
                        dest
                    }
                    PostfixOp::Deref => {
                        let ty = Type::Deref(ctx.b.infer());
                        let dest = ctx.temp(ty);
                        ctx.push(UInstruction::Deref { dst: dest, src: res });
                        dest
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
                    dst: VarInst { status: , origin: () },
                    f: fe,
                    args: nargs,
                });
                dest
            }
            PExpr::Group(e) => e.lower(ctx)?,
            PExpr::Construct(e, map) => {
                let dest = ctx.temp(Type::Placeholder);
                ctx.push(UInstruction::Construct { dst: dest, fields: () });
                dest
            }
            PExpr::If(cond, body) => {
                let cond = cond.lower(ctx)?;
                ctx.b.push();
                let mut body_ctx = ctx.branch();
                body.lower(&mut body_ctx);
                let body = body_ctx.instructions;
                ctx.b.pop();
                ctx.push(UInstruction::If { cond, body });
                return None;
            }
            PExpr::Loop(body) => {
                ctx.b.push();
                let mut body_ctx = ctx.branch();
                body.lower(&mut body_ctx);
                let body = body_ctx.instructions;
                ctx.b.pop();
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
            PExpr::Member(e, name) => {
                ctx.err("Can't access a member here".to_string());
                return None;
            }
            PExpr::Field(e, name) => {
                todo!()
            }
        })
    }
}
