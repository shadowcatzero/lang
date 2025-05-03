use std::collections::HashMap;

use super::{CompilerMsg, CompilerOutput, FileSpan, FnLowerable, Imports, Node, PFunction};
use crate::{
    ir::{
        FnID, Origin, Typable, Type, UFunc, UInstrInst, UInstruction, UModuleBuilder, UVar, VarID,
        VarInst, VarStatus,
    },
    parser, util::NameStack,
};

impl Node<PFunction> {
    pub fn lower(
        &self,
        b: &mut UModuleBuilder,
        imports: &mut Imports,
        output: &mut CompilerOutput,
    ) -> Option<FnID> {
        self.as_ref()
            .map(|s| s.lower(b, imports, output, self.origin))
            .flatten()
    }
}

impl PFunction {
    pub fn lower(
        &self,
        b: &mut UModuleBuilder,
        imports: &mut Imports,
        output: &mut CompilerOutput,
        origin: Origin,
    ) -> Option<FnID> {
        let header = self.header.as_ref()?;
        let name = header.name.as_ref()?.0.clone();
        let (generic_args, args, ret) = if let Some(header) = self.header.as_ref() {
            (
                header
                    .gargs
                    .iter()
                    .map(|a| Some(a.lower(b, output)))
                    .collect(),
                header
                    .args
                    .iter()
                    .flat_map(|a| Some(a.lower(b, output)?))
                    .collect(),
                match &header.ret {
                    Some(ty) => ty.lower(b, output),
                    None => b.def_ty(Type::Unit),
                },
            )
        } else {
            (Vec::new(), Vec::new(), b.error)
        };
        let instructions = {
            let mut var_stack = Vec::new();
            let mut ctx = FnLowerCtx {
                instructions: Vec::new(),
                var_stack: &mut var_stack,
                b,
                output,
                origin: self.body.origin,
                imports,
            };
            if let Some(src) = self.body.lower(&mut ctx) {
                ctx.instructions.push(UInstrInst {
                    origin: src.origin,
                    i: UInstruction::Ret { src },
                });
            }
            ctx.instructions
        };
        let gargs = args.iter().map(|a| b.vars[a].ty).collect();
        let f = UFunc {
            origin,
            gargs,
            name,
            args,
            ret,
            instructions,
        };
        Some(b.def_fn(f))
    }
}

pub struct FnLowerCtx<'a, 'b> {
    pub b: &'a mut UModuleBuilder<'b>,
    pub instructions: Vec<UInstrInst>,
    pub output: &'a mut CompilerOutput,
    pub origin: FileSpan,
    pub imports: &'a mut Imports,
    pub var_stack: &'a mut NameStack<VarID>,
}

impl<'a, 'b> FnLowerCtx<'a, 'b> {
    pub fn var(&mut self, node: &Node<parser::PIdent>) -> VarInst {
        if let Some(n) = node.as_ref() {
            if let Some(&var) = self.var_stack.search(&n.0) {
                return VarInst {
                    status: VarStatus::Res(var),
                    origin: node.origin,
                }
            }
        }
    }
    pub fn err(&mut self, msg: String) {
        self.output.err(CompilerMsg::new(msg, self.origin))
    }
    pub fn err_at(&mut self, span: FileSpan, msg: String) {
        self.output.err(CompilerMsg::new(msg, span))
    }
    pub fn temp<T: Typable>(&mut self, ty: Type) -> VarInst {
        self.b.temp_var(self.origin, ty)
    }
    pub fn push(&mut self, i: UInstruction) {
        self.push_at(i, self.origin);
    }
    pub fn push_at(&mut self, i: UInstruction, span: FileSpan) {
        self.instructions.push(UInstrInst { i, origin: span });
    }
    pub fn branch(&'a mut self) -> FnLowerCtx<'a, 'b> {
        FnLowerCtx {
            b: self.b,
            instructions: Vec::new(),
            var_stack: self.var_stack,
            output: self.output,
            origin: self.origin,
            imports: self.imports,
        }
    }
}
