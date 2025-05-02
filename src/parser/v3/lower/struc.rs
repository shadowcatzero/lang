use crate::{
    common::{CompilerOutput, FileSpan},
    ir::{StructField, StructID, UInstruction, UModuleBuilder, UProgram, UStruct, VarInst},
    parser::{Node, PMap, PStruct, PStructFields},
};

use super::{FnLowerCtx, FnLowerable};

impl FnLowerable for PMap {
    type Output = VarInst;
    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<VarInst> {
        let ty = self.name.lower(ctx.b, ctx.output);
        let fields = match &self.fields {
            PConstructFields::Named(nodes) => nodes
                .iter()
                .flat_map(|n| {
                    let def = n.as_ref()?;
                    let name = def.name.as_ref()?.to_string();
                    let expr = def.val.as_ref()?.lower(ctx)?;
                    Some((name, expr))
                })
                .collect(),
            PConstructFields::Tuple(nodes) => nodes
                .iter()
                .enumerate()
                .flat_map(|(i, n)| {
                    let expr = n.as_ref()?.lower(ctx)?;
                    let name = format!("{i}");
                    Some((name, expr))
                })
                .collect(),
            PConstructFields::None => Default::default(),
        };
        let id = ctx.temp(ty);
        ctx.push(UInstruction::Construct { dst: id, fields });
        Some(id)
    }
}

impl PStruct {
    pub fn lower(
        &self,
        id: StructID,
        p: &mut UModuleBuilder,
        output: &mut CompilerOutput,
        span: FileSpan,
    ) -> Option<()> {
        p.push();
        let generics = self.generics.iter().flat_map(|a| a.lower(p)).collect();
        let fields = match &self.fields {
            PStructFields::Named(nodes) => nodes
                .iter()
                .flat_map(|n| {
                    let def = n.as_ref()?;
                    let name = def.name.as_ref()?.to_string();
                    let tynode = def.ty.as_ref()?;
                    let ty = tynode.lower(p, output);
                    Some((name, ty))
                })
                .collect(),
            PStructFields::Tuple(nodes) => nodes
                .iter()
                .enumerate()
                .flat_map(|(i, n)| {
                    let ty = n.as_ref()?.lower(p, output, span);
                    Some((format!("{i}"), ty))
                })
                .collect(),
            PStructFields::None => vec![],
        }
        .into_iter()
        .map(|(name, ty)| (name, StructField { ty }))
        .collect();
        let name = self.name.as_ref()?.to_string();
        p.def_data(UStruct {
            name,
            generics,
            fields,
            origin: span,
        });
        p.pop();
        Some(())
    }
}

impl Node<PStruct> {
    pub fn lower(&self, id: StructID, p: &mut UProgram, output: &mut CompilerOutput) -> Option<()> {
        let s = self.as_ref()?;
        let name = s.name.as_ref()?;
        s.lower(id, p, output, self.origin);
        Some(())
    }
}
