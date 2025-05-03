use crate::ir::{UProgram, UVar, VarID, VarInst};

use super::{CompilerOutput, Node, PVarDef};

impl Node<PVarDef> {
    pub fn lower(&self, program: &mut UProgram, output: &mut CompilerOutput) -> Option<VarID> {
        let s = self.as_ref()?;
        let name = s.name.as_ref().map_or("{error}", |v| v);
        let ty = match &s.ty {
            Some(ty) => ty.lower(program, output),
            None => program.infer(self.origin),
        };
        Some(VarInst {
            id: program.def_searchable(name, Some(UVar { ty }), self.origin),
            origin: self.origin,
        })
    }
}
