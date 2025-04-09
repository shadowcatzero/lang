use std::collections::HashMap;

use crate::{
    common::{CompilerOutput, FileSpan},
    ir::{FieldID, IRUInstruction, IRUProgram, StructDef, StructField, Type, VarInst},
    parser::{Node, PConstruct, PConstructFields, PStruct, PStructFields},
};

use super::{FnLowerCtx, FnLowerable};

impl FnLowerable for PConstruct {
    type Output = VarInst;
    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<VarInst> {
        let ty = self.name.lower(ctx.program, ctx.output);
        let field_map = match ty {
            Type::Struct { id, .. } => ctx.program.get_struct(id),
            _ => {
                ctx.err(format!(
                    "Type {} cannot be constructed",
                    ctx.program.type_name(&ty)
                ));
                return None;
            }
        }
        .field_map
        .clone();
        let fields = match &self.fields {
            PConstructFields::Named(nodes) => nodes
                .iter()
                .flat_map(|n| {
                    let def = n.as_ref()?;
                    let name = def.name.as_ref()?.to_string();
                    let expr = def.val.as_ref()?.lower(ctx)?;
                    let Some(&field) = field_map.get(&name) else {
                        ctx.err(format!(
                            "Struct {} has no field {}",
                            ctx.program.type_name(&ty),
                            name
                        ));
                        return None;
                    };
                    Some((field, expr))
                })
                .collect(),
            PConstructFields::Tuple(nodes) => nodes
                .iter()
                .enumerate()
                .flat_map(|(i, n)| {
                    let expr = n.as_ref()?.lower(ctx)?;
                    let name = format!("{i}");
                    let Some(&field) = field_map.get(&name) else {
                        ctx.err(format!(
                            "Struct {} has no field {}",
                            ctx.program.type_name(&ty),
                            name
                        ));
                        return None;
                    };
                    Some((field, expr))
                })
                .collect(),
            PConstructFields::None => Default::default(),
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
        let mut field_map = HashMap::new();
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
        .enumerate()
        .map(|(i, (name, ty))| {
            let id = FieldID(i);
            field_map.insert(name.clone(), id);
            StructField { name, ty }
        })
        .collect();
        p.def_struct(StructDef {
            name: self.name.as_ref()?.to_string(),
            origin: span,
            field_map,
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
