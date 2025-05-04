use crate::{
    common::{CompilerOutput, FileSpan},
    ir::{StructField, StructID, UModuleBuilder, UProgram, UStruct},
    parser::{Node, PStruct, PStructFields},
};

impl PStruct {
    pub fn lower(
        &self,
        id: StructID,
        b: &mut UModuleBuilder,
        output: &mut CompilerOutput,
        span: FileSpan,
    ) -> Option<()> {
        let generics = self.generics.iter().flat_map(|a| a.lower(b)).collect();
        let fields = match &self.fields {
            PStructFields::Named(nodes) => nodes
                .iter()
                .flat_map(|n| {
                    let def = n.as_ref()?;
                    let name = def.name.as_ref()?.to_string();
                    let tynode = def.ty.as_ref()?;
                    let ty = tynode.lower(b, output);
                    Some((name, ty))
                })
                .collect(),
            PStructFields::Tuple(nodes) => nodes
                .iter()
                .enumerate()
                .flat_map(|(i, n)| {
                    let ty = n.as_ref()?.lower(b, output, span);
                    Some((format!("{i}"), ty))
                })
                .collect(),
            PStructFields::None => vec![],
        }
        .into_iter()
        .map(|(name, ty)| (name, StructField { ty }))
        .collect();
        let name = self.name.as_ref()?.to_string();
        b.def_data(UStruct {
            name,
            gargs: generics,
            fields,
            origin: span,
        });
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
