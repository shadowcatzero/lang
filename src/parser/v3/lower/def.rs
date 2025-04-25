use crate::ir::{Type, UProgram, UVar, VarInst};

use super::{CompilerOutput, Node, PVarDef};

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
            id: program.def_searchable(name, Some(UVar { ty, parent: None }), self.origin),
            span: self.origin,
        })
    }
}
