use crate::ir::Instruction;

use super::{Block, ExprResult, FnLowerCtx, Node, Statement};

impl Node<Block> {
    pub fn lower(&self, ctx: &mut FnLowerCtx) -> Option<ExprResult> {
        self.as_ref()?.lower(ctx)
    }
}

impl Block {
    pub fn lower(&self, ctx: &mut FnLowerCtx) -> Option<ExprResult> {
        let ctx = &mut ctx.sub();
        for statement in &self.statements {
            statement.lower(ctx);
        }
        self.result.as_ref()?.lower(ctx)
    }
}

impl Node<Box<Statement>> {
    pub fn lower(&self, ctx: &mut FnLowerCtx) -> Option<ExprResult> {
        self.as_ref()?.lower(ctx)
    }
}

impl Node<Statement> {
    pub fn lower(&self, ctx: &mut FnLowerCtx) -> Option<ExprResult> {
        self.as_ref()?.lower(ctx)
    }
}

impl Statement {
    pub fn lower(&self, ctx: &mut FnLowerCtx) -> Option<ExprResult> {
        match self {
            super::Statement::Let(def, e) => {
                let def = def.lower(ctx.map, ctx.output)?;
                let res = e.lower(ctx);
                if let Some(res) = res {
                    match res {
                        ExprResult::Var(v) => ctx.map.name_var(&def, v),
                        ExprResult::Fn(_) => todo!(),
                    }
                }
                None
            }
            super::Statement::Return(e) => {
                let res = e.lower(ctx)?;
                match res {
                    ExprResult::Var(v) => ctx.push(Instruction::Ret { src: v }),
                    _ => todo!(),
                }
                None
            }
            super::Statement::Expr(e) => e.lower(ctx),
        }
    }
}
