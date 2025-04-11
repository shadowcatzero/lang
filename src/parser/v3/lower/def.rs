use crate::ir::{Type, UProgram, UStruct, UVar, VarInst};

use super::{CompilerMsg, CompilerOutput, FileSpan, Node, PType, PVarDef};

impl Node<PVarDef> {
    pub fn lower(&self, program: &mut UProgram, output: &mut CompilerOutput) -> Option<VarInst> {
        let s = self.as_ref()?;
        let name = s
            .name
            .as_ref()
            .map(|n| n.to_string())
            .unwrap_or("{error}".to_string());
        let ty = match &s.ty {
            Some(ty) => ty.lower(program, output),
            None => Type::Infer,
        };
        Some(VarInst {
            id: program.def_searchable(
                name,
                Some(UVar {
                    ty,
                    parent: None,
                    origin: self.span,
                }),
            ),
            span: self.span,
        })
    }
}

impl Node<PType> {
    pub fn lower(&self, namespace: &mut UProgram, output: &mut CompilerOutput) -> Type {
        self.as_ref()
            .map(|t| t.lower(namespace, output, self.span))
            .unwrap_or(Type::Error)
    }
}

impl PType {
    pub fn lower(
        &self,
        namespace: &mut UProgram,
        output: &mut CompilerOutput,
        span: FileSpan,
    ) -> Type {
        let Some(name) = self.name.as_ref() else {
            return Type::Error;
        };
        match namespace
            .get_idents(name)
            .and_then(|ids| ids.get::<UStruct>())
        {
            Some(id) => {
                let args = self
                    .args
                    .iter()
                    .map(|n| n.lower(namespace, output))
                    .collect();
                Type::Struct { id, args }
            }
            None => {
                if let Ok(num) = name.parse::<u32>() {
                    Type::Bits(num)
                } else {
                    match name.as_str() {
                        "slice" => {
                            let inner = self.args[0].lower(namespace, output);
                            Type::Slice(Box::new(inner))
                        }
                        "_" => Type::Infer,
                        _ => {
                            output.err(CompilerMsg::from_span(span, "Type not found".to_string()));
                            Type::Error
                        }
                    }
                }
            }
        }
    }
}
