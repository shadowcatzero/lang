use crate::{
    common::{CompilerOutput, FileSpan},
    ir::{StructField, StructID, UInstruction, UProgram, UStruct, VarInst},
    parser::{Node, PConstruct, PConstructFields, PStruct, PStructFields},
};

use super::{FnLowerCtx, FnLowerable};

impl FnLowerable for PConstruct {
    type Output = VarInst;
    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<VarInst> {
        let ty = self.name.lower(ctx.program, ctx.output);
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
        ctx.push(UInstruction::Construct { dest: id, fields });
        Some(id)
    }
}

impl PStruct {
    pub fn lower(
        &self,
        id: StructID,
        p: &mut UProgram,
        output: &mut CompilerOutput,
        span: FileSpan,
    ) -> Option<()> {
        p.push();
        let generics = self
            .generics
            .iter()
            .flat_map(|a| a.lower(p))
            .collect();
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
        p.write(id, UStruct { generics, fields });
        p.pop();
        Some(())
    }
}

impl Node<PStruct> {
    pub fn lower_name(&self, p: &mut UProgram) -> Option<StructID> {
        let s = self.as_ref()?;
        let name = s.name.as_ref()?.to_string();
        let id = p.def_searchable(name.to_string(), None, s.name.span);
        Some(id)
    }
    pub fn lower(&self, id: StructID, p: &mut UProgram, output: &mut CompilerOutput) {
        self.as_ref().map(|i| i.lower(id, p, output, self.span));
    }
}
