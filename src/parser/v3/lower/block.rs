use crate::ir::{IRUInstruction, VarID};

use super::{PBlock, FnLowerCtx, FnLowerable, PStatement};

impl FnLowerable for PBlock {
    type Output = VarID;
    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<VarID> {
        let ctx = &mut ctx.sub();
        for statement in &self.statements {
            statement.lower(ctx);
        }
        self.result.as_ref()?.lower(ctx)
    }
}

impl FnLowerable for PStatement {
    type Output = VarID;
    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<VarID> {
        match self {
            super::PStatement::Let(def, e) => {
                let def = def.lower(ctx.map, ctx.output)?;
                let res = e.lower(ctx);
                if let Some(res) = res {
                    ctx.map.name_var(&def, res);
                }
                None
            }
            super::PStatement::Return(e) => {
                let src = e.lower(ctx)?;
                ctx.push(IRUInstruction::Ret { src });
                None
            }
            super::PStatement::Expr(e) => e.lower(ctx),
        }
    }
}
