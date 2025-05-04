use std::collections::HashMap;

use crate::{ir::VarInstID, parser::PMap};

use super::{FnLowerCtx, FnLowerable};

impl FnLowerable for PMap {
    type Output = HashMap<String, VarInstID>;
    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<Self::Output> {
        Some(
            self.0
                .iter()
                .flat_map(|n| {
                    let def = n.as_ref()?;
                    let name = def.name.as_ref()?.to_string();
                    let expr = def.val.as_ref()?.lower(ctx)?;
                    Some((name, expr))
                })
                .collect(),
        )
    }
}
