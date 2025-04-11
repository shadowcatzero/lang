use crate::ir::{Type, UInstruction, VarInst};

use super::{FnLowerCtx, FnLowerable, PBlock, PStatement};

impl FnLowerable for PBlock {
    type Output = VarInst;
    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<VarInst> {
        ctx.program.push();
        for statement in &self.statements {
            statement.lower(ctx);
        }
        let res = self.result.as_ref().and_then(|r| r.lower(ctx));
        ctx.program.pop();
        res
    }
}

impl FnLowerable for PStatement {
    type Output = VarInst;
    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<VarInst> {
        match self {
            super::PStatement::Let(def, e) => {
                let def = def.lower(ctx.program, ctx.output)?;
                let res = e.lower(ctx);
                if let Some(res) = res {
                    ctx.push(UInstruction::Mv {
                        dest: def,
                        src: res,
                    });
                }
                None
            }
            super::PStatement::Return(e) => {
                if let Some(e) = e {
                    let src = e.lower(ctx)?;
                    ctx.push_at(UInstruction::Ret { src }, src.span);
                } else {
                    let src = ctx.temp(Type::Unit);
                    ctx.push_at(UInstruction::Ret { src }, src.span);
                }
                None
            }
            super::PStatement::Expr(e) => e.lower(ctx),
        }
    }
}
