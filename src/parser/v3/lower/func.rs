use std::collections::HashMap;

use super::{CompilerMsg, CompilerOutput, FileSpan, FnLowerable, Imports, Node, PFunction};
use crate::{
    ir::{
        FnID, Idents, Typable, Type, UFunc, UInstrInst, UInstruction, UModuleBuilder, UProgram,
        UVar, VarID, VarInst,
    },
    parser,
};

impl Node<PFunction> {
    pub fn lower_name(&self, p: &mut UProgram) -> Option<FnID> {
        self.as_ref()?.lower_name(p)
    }
    pub fn lower(
        &self,
        id: FnID,
        p: &mut UProgram,
        imports: &mut Imports,
        output: &mut CompilerOutput,
    ) {
        if let Some(s) = self.as_ref() {
            s.lower(id, p, imports, output)
        }
    }
}

impl PFunction {
    pub fn lower_name(&self, p: &mut UProgram) -> Option<FnID> {
        let header = self.header.as_ref()?;
        let name = header.name.as_ref()?;
        let id = p.def_searchable(name, None, self.header.origin);
        Some(id)
    }
    pub fn lower(
        &self,
        id: FnID,
        p: &mut UProgram,
        imports: &mut Imports,
        output: &mut CompilerOutput,
    ) {
        let name = p.names.name(id).to_string();
        p.push_name(&name);
        let (args, ret) = if let Some(header) = self.header.as_ref() {
            (
                header
                    .args
                    .iter()
                    .flat_map(|a| Some(a.lower(p, output)?.id))
                    .collect(),
                match &header.ret {
                    Some(ty) => ty.lower(p, output),
                    None => Type::Unit,
                },
            )
        } else {
            (Vec::new(), Type::Error)
        };
        let mut ctx = FnLowerCtx {
            instructions: Vec::new(),
            b: p,
            output,
            origin: self.body.origin,
            imports,
        };
        if let Some(src) = self.body.lower(&mut ctx) {
            ctx.instructions.push(UInstrInst {
                i: UInstruction::Ret { src },
                origin: src.origin,
            });
        }
        let instructions = ctx.instructions;
        let f = UFunc {
            args,
            ret,
            instructions,
        };
        p.pop_name();
        p.write(id, f)
    }
}

pub struct FnLowerCtx<'a> {
    pub b: &'a mut UModuleBuilder<'a>,
    pub instructions: Vec<UInstrInst>,
    pub output: &'a mut CompilerOutput,
    pub origin: FileSpan,
    pub imports: &'a mut Imports,
    pub var_stack: Vec<HashMap<String, VarID>>,
}

impl FnLowerCtx<'_> {
    pub fn get_idents(&mut self, node: &Node<parser::PIdent>) -> Option<Idents> {
        let name = node.inner.as_ref()?;
        let res = self.b.get_idents(name);
        if res.is_none() {
            self.err_at(node.origin, format!("Identifier '{}' not found", name));
        }
        res
    }
    pub fn get_var(&mut self, node: &Node<parser::PIdent>) -> VarInst {
        let ids = self.get_idents(node)?;
        if ids.get::<UVar>().is_none() {
            self.err_at(
                node.origin,
                format!("Variable '{}' not found", node.inner.as_ref()?),
            );
        }
        ids.get::<UVar>().map(|id| VarInst {
            id,
            origin: node.origin,
        })
    }
    pub fn err(&mut self, msg: String) {
        self.output.err(CompilerMsg::new(self.origin, msg))
    }
    pub fn err_at(&mut self, span: FileSpan, msg: String) {
        self.output.err(CompilerMsg::new(span, msg))
    }
    pub fn temp<T: Typable>(&mut self, ty: Type) -> VarInst {
        self.b.temp_var(self.origin, ty)
    }
    pub fn push(&mut self, i: UInstruction) {
        self.push_at(i, self.origin);
    }
    pub fn push_at(&mut self, i: UInstruction, span: FileSpan) {
        match i {
            UInstruction::Mv { dst: dest, src } => todo!(),
            UInstruction::Ref { dst: dest, src } => todo!(),
            UInstruction::LoadData { dst: dest, src } => todo!(),
            UInstruction::LoadSlice { dst: dest, src } => todo!(),
            UInstruction::LoadFn { dst: dest, src } => todo!(),
            UInstruction::Call { dst: dest, f, args } => todo!(),
            UInstruction::AsmBlock { instructions, args } => todo!(),
            UInstruction::Ret { src } => todo!(),
            UInstruction::Construct { dst: dest, fields } => todo!(),
            UInstruction::If { cond, body } => todo!(),
            UInstruction::Loop { body } => todo!(),
            UInstruction::Break => todo!(),
            UInstruction::Continue => todo!(),
        }
        self.instructions.push(UInstrInst { i, origin: span });
    }
    pub fn branch<'a>(&'a mut self) -> FnLowerCtx<'a> {
        FnLowerCtx {
            b: self.b,
            instructions: Vec::new(),
            output: self.output,
            origin: self.origin,
            imports: self.imports,
        }
    }
}
