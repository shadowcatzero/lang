use super::{func::FnLowerCtx, Expr as PExpr, Node, UnaryOp};
use crate::ir::{FnIdent, Instruction, Type, VarIdent, VarOrFnIdent};

impl PExpr {
    pub fn lower(&self, ctx: &mut FnLowerCtx) -> Option<ExprResult> {
        Some(match self {
            PExpr::Lit(l) => match l.as_ref()? {
                super::Literal::String(s) => todo!(),
                super::Literal::Char(c) => todo!(),
                super::Literal::Number(n) => todo!(),
                super::Literal::Unit => {
                    todo!();
                }
            },
            PExpr::Ident(i) => {
                let name = i.as_ref()?.val();
                let Some(id) = ctx.get(name) else {
                    ctx.err(format!("Identifier '{}' not found.", name));
                    return None;
                };
                let Some(vf) = id.var_func else {
                    ctx.err(format!("Variable or function '{}' not found; Found type, but types cannot be used here.", name));
                    return None;
                };
                match vf {
                    VarOrFnIdent::Var(var) => ExprResult::Var(var),
                    VarOrFnIdent::Fn(f) => ExprResult::Fn(f),
                }
            }
            PExpr::BinaryOp(op, e1, e2) => {
                let res1 = e1.lower(ctx)?;
                let res2 = e2.lower(ctx)?;
                op.traitt();
                todo!();
            }
            PExpr::UnaryOp(op, e) => {
                let res = e.lower(ctx)?;
                match op {
                    UnaryOp::Ref => ExprResult::Var(match res {
                        ExprResult::Var(v) => {
                            let temp = ctx.temp(ctx.map.get_var(v).ty.clone());
                            ctx.push(Instruction::Ref { dest: temp, src: v });
                            temp
                        }
                        ExprResult::Fn(f) => {
                            let temp = ctx.temp(Type::Ref(Box::new(ctx.map.get_fn(f).ty())));
                            ctx.push(Instruction::Lf { dest: temp, src: f });
                            temp
                        }
                    }),
                    UnaryOp::Deref => match res {
                        ExprResult::Var(v) => match &ctx.map.get_var(v).ty {
                            Type::Ref(inner) => {
                                todo!()
                            }
                            t => {
                                ctx.err(format!(
                                    "Cannot dereference type {:?}",
                                    ctx.map.type_name(t)
                                ));
                                return None;
                            }
                        },
                        ExprResult::Fn(f) => {
                            ctx.err("Cannot dereference functions".to_string());
                            return None;
                        }
                    },
                    UnaryOp::Not => todo!(),
                }
            }
            PExpr::Block(b) => b.lower(ctx)?,
            PExpr::AsmBlock(b) => {
                ctx.push(Instruction::AsmBlock {
                    instructions: b.as_ref()?.instructions.clone(),
                });
                return None;
            }
            PExpr::Call(e, args) => {
                let fe = e.lower(ctx)?;
                let mut nargs = Vec::new();
                for arg in args.iter() {
                    let arg = arg.lower(ctx)?;
                    nargs.push(match arg {
                        ExprResult::Var(v) => v,
                        ExprResult::Fn(_) => todo!(),
                    });
                }
                match fe {
                    ExprResult::Fn(f) => {
                        let temp = ctx.temp(ctx.map.get_fn(f).ret.clone());
                        ctx.push(Instruction::Call {
                            dest: temp,
                            f,
                            args: nargs,
                        });
                        ExprResult::Var(temp)
                    }
                    o => {
                        ctx.err(format!("Expected function, found {:?}", o));
                        return None;
                    }
                }
            }
            PExpr::Group(e) => e.lower(ctx)?,
        })
    }
}

impl Node<PExpr> {
    pub fn lower(&self, ctx: &mut FnLowerCtx) -> Option<ExprResult> {
        self.inner.as_ref()?.lower(&mut FnLowerCtx {
            map: ctx.map,
            instructions: ctx.instructions,
            output: ctx.output,
            span: self.span,
        })
    }
}

impl Node<Box<PExpr>> {
    pub fn lower(&self, ctx: &mut FnLowerCtx) -> Option<ExprResult> {
        self.inner.as_ref()?.lower(&mut FnLowerCtx {
            map: ctx.map,
            instructions: ctx.instructions,
            output: ctx.output,
            span: self.span,
        })
    }
}

#[derive(Debug)]
pub enum ExprResult {
    Var(VarIdent),
    Fn(FnIdent),
}
