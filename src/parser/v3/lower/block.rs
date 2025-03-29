use crate::ir::{IRUInstruction, VarInst};

use super::{FnLowerCtx, FnLowerable, PBlock, PStatement};

impl FnLowerable for PBlock {
    type Output = VarInst;
    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<VarInst> {
        ctx.program.push();
        for statement in &self.statements {
            statement.lower(ctx);
        }
        let res = self.result.as_ref().map(|r| r.lower(ctx)).flatten();
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
                    ctx.program.name_var(&def, res.id);
                }
                None
            }
            super::PStatement::Return(e) => {
                let src = e.lower(ctx)?;
                ctx.push_at(IRUInstruction::Ret { src }, src.span);
                None
            }
            super::PStatement::Expr(e) => e.lower(ctx),
        }
    }
}
