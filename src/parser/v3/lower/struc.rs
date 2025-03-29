use std::collections::HashMap;

use crate::{
    common::{CompilerMsg, CompilerOutput, FileSpan},
    ir::{IRUInstruction, IRUProgram, Origin, StructDef, StructField, VarInst},
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
                    Some((format!("{i}"), expr))
                })
                .collect(),
            PConstructFields::None => HashMap::new(),
        };
        let id = ctx.temp(ty);
        ctx.push(IRUInstruction::Construct { dest: id, fields });
        Some(id)
    }
}

impl PStruct {
    pub fn lower(
        &self,
        p: &mut IRUProgram,
        output: &mut CompilerOutput,
        span: FileSpan,
    ) -> Option<()> {
        let mut offset = 0;
        let fields = match &self.fields {
            PStructFields::Named(nodes) => nodes
                .iter()
                .flat_map(|n| {
                    let def = n.as_ref()?;
                    let name = def.name.as_ref()?.to_string();
                    let tynode = def.ty.as_ref()?;
                    let ty = tynode.lower(p, output);
                    let size = p.size_of_type(&ty).unwrap_or_else(|| {
                        output.err(CompilerMsg {
                            msg: format!("Size of type '{}' unknown", p.type_name(&ty)),
                            spans: vec![tynode.span],
                        });
                        0
                    });
                    let res = Some((name, StructField { ty, offset }));
                    offset += size;
                    res
                })
                .collect(),
            PStructFields::Tuple(nodes) => nodes
                .iter()
                .enumerate()
                .flat_map(|(i, n)| {
                    let ty = n.as_ref()?.lower(p, output, span);
                    let size = p.size_of_type(&ty)?;
                    let res = Some((format!("{i}"), StructField { ty, offset }));
                    offset += size;
                    res
                })
                .collect(),
            PStructFields::None => HashMap::new(),
        };
        p.def_type(StructDef {
            name: self.name.as_ref()?.to_string(),
            origin: Origin::File(span),
            size: offset,
            fields,
        });
        Some(())
    }
}

impl Node<PStruct> {
    pub fn lower(&self, p: &mut IRUProgram, output: &mut CompilerOutput) {
        self.as_ref().map(|i| i.lower(p, output, self.span));
    }
}
