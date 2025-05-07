use std::collections::HashMap;

use crate::ir::{UVar, VarID};

use super::{ModuleLowerCtx, Node, PVarDef};

impl Node<PVarDef> {
    pub fn lower(&self, ctx: &mut ModuleLowerCtx) -> Option<VarID> {
        let s = self.as_ref()?;
        let name = s.name.as_ref().map_or("{error}", |v| v).to_string();
        let ty = match &s.ty {
            Some(ty) => ty.lower(ctx),
            None => ctx.infer(),
        };
        Some(ctx.def_var(UVar {
            name,
            ty,
            origin: self.origin,
            parent: None,
            children: HashMap::new(),
        }))
    }
}
