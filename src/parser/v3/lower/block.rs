use crate::{
    ir::{Type, UInstruction, VarInst},
    parser::{PConstStatement, PStatementLike},
};

use super::{FnLowerCtx, FnLowerable, PBlock, PStatement};

impl FnLowerable for PBlock {
    type Output = VarInst;
    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<VarInst> {
        ctx.program.push();
        let mut last = None;
        let mut statements = Vec::new();
        let mut fn_nodes = Vec::new();
        let mut struct_nodes = Vec::new();
        // first sort statements
        for s in &self.statements {
            let Some(s) = s.as_ref() else {
                continue;
            };
            match s {
                PStatementLike::Statement(s) => statements.push(s),
                PStatementLike::Const(pconst_statement) => match pconst_statement {
                    PConstStatement::Fn(f) => fn_nodes.push(f),
                    PConstStatement::Struct(s) => struct_nodes.push(s),
                },
            }
        }
        // then lower const things
        let mut structs = Vec::new();
        for s in &struct_nodes {
            structs.push(s.lower_name(ctx.program));
        }
        for (s, id) in struct_nodes.iter().zip(structs) {
            if let Some(id) = id {
                s.lower(id, ctx.program, ctx.output);
            }
        }
        let mut fns = Vec::new();
        for f in &fn_nodes {
            fns.push(f.lower_name(ctx.program));
        }
        for (f, id) in fn_nodes.iter().zip(fns) {
            if let Some(id) = id {
                f.lower(id, ctx.program, ctx.output)
            }
        }
        // then lower statements
        for s in statements {
            last = s.lower(ctx);
        }
        ctx.program.pop();
        last
    }
}

impl FnLowerable for PStatement {
    type Output = VarInst;
    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<VarInst> {
        match self {
            PStatement::Let(def, e) => {
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
            PStatement::Return(e) => {
                if let Some(e) = e {
                    let src = e.lower(ctx)?;
                    ctx.push_at(UInstruction::Ret { src }, src.span);
                } else {
                    let src = ctx.temp(Type::Unit);
                    ctx.push_at(UInstruction::Ret { src }, src.span);
                }
                None
            }
            PStatement::Expr(e) => e.lower(ctx),
        }
    }
}
