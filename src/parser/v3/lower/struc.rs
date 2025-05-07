use crate::{
    common::FileSpan,
    ir::{StructField, StructID, UStruct},
    parser::{PStruct, PStructFields},
};

use super::ModuleLowerCtx;

impl PStruct {
    pub fn lower(&self, ctx: &mut ModuleLowerCtx, span: FileSpan) -> Option<StructID> {
        ctx.ident_stack.push();
        let gmap: Vec<_> = self.generics.iter().flat_map(|a| a.lower(ctx)).collect();
        let gargs = gmap.iter().map(|(_, id)| *id).collect();
        let fields = match &self.fields {
            PStructFields::Named(nodes) => nodes
                .iter()
                .flat_map(|n| {
                    let def = n.as_ref()?;
                    let name = def.name.as_ref()?.to_string();
                    let tynode = def.ty.as_ref()?;
                    let ty = tynode.lower(ctx);
                    Some((name, ty))
                })
                .collect(),
            PStructFields::Tuple(nodes) => nodes
                .iter()
                .enumerate()
                .flat_map(|(i, n)| {
                    let ty = n.as_ref()?.lower(ctx, span);
                    Some((format!("{i}"), ty))
                })
                .collect(),
            PStructFields::None => vec![],
        }
        .into_iter()
        .map(|(name, ty)| (name, StructField { ty }))
        .collect();
        let name = self.name.as_ref()?.to_string();
        ctx.ident_stack.pop();
        Some(ctx.def_struct(UStruct {
            name,
            gargs,
            fields,
            origin: span,
        }))
    }
}
