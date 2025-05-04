use crate::{
    ir::{Type, UInstruction, UVar, VarInst, VarInstID},
    parser::{PConstStatement, PStatementLike},
};

use super::{FnLowerCtx, FnLowerable, Import, PBlock, PStatement};

impl FnLowerable for PBlock {
    type Output = VarInstID;
    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<VarInstID> {
        let mut last = None;
        let mut statements = Vec::new();
        let mut fn_nodes = Vec::new();
        let mut struct_nodes = Vec::new();
        let mut import_nodes = Vec::new();
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
                    PConstStatement::Import(i) => import_nodes.push(i),
                },
            }
        }
        ctx.b.push();
        // then lower imports
        for i_n in &import_nodes {
            if let Some(i) = i_n.as_ref() {
                let name = &i.0;
                let path = ctx.b.path_for(name);
                let import = Import(path.clone());
                if ctx.imports.insert(import) {
                    ctx.b.def_searchable::<UVar>(
                        name,
                        Some(UVar {
                            ty: Type::Module(path),
                        }),
                        i_n.origin,
                    );
                }
            }
        }
        // then lower const things
        let mut structs = Vec::new();
        for s in &struct_nodes {
            structs.push(s.lower_name(ctx.b));
        }
        for (s, id) in struct_nodes.iter().zip(structs) {
            if let Some(id) = id {
                s.lower(id, ctx.b, ctx.output);
            }
        }
        let mut fns = Vec::new();
        for f in &fn_nodes {
            fns.push(f.lower_name(ctx.b));
        }
        for (f, id) in fn_nodes.iter().zip(fns) {
            if let Some(id) = id {
                f.lower(id, ctx.b, ctx.imports, ctx.output)
            }
        }
        // then lower statements
        for s in statements {
            last = s.lower(ctx);
        }
        ctx.b.pop();
        last
    }
}

impl FnLowerable for PStatement {
    type Output = VarInst;
    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<VarInst> {
        match self {
            PStatement::Let(def, e) => {
                let def = def.lower(ctx.b, ctx.output)?;
                let res = e.lower(ctx);
                if let Some(res) = res {
                    ctx.push(UInstruction::Mv {
                        dst: def,
                        src: res,
                    });
                }
                None
            }
            PStatement::Return(e) => {
                if let Some(e) = e {
                    let src = e.lower(ctx)?;
                    ctx.push_at(UInstruction::Ret { src }, src.origin);
                } else {
                    let src = ctx.temp(Type::Unit);
                    ctx.push_at(UInstruction::Ret { src }, src.origin);
                }
                None
            }
            PStatement::Expr(e) => e.lower(ctx),
        }
    }
}
