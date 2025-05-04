use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use super::{CompilerMsg, CompilerOutput, FileSpan, FnLowerable, Imports, Node, PFunction};
use crate::{
    ir::{
        FnID, GenericID, ModPath, Origin, Typable, Type, UFunc, UInstrInst, UInstruction,
        UModuleBuilder, VarID, VarInst, VarInstID, VarStatus,
    },
    parser,
    util::NameStack,
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
        let (generics, args, ret) = if let Some(header) = self.header.as_ref() {
            (
                header.gargs.iter().flat_map(|a| a.lower(b)).collect(),
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
            (Vec::new(), Vec::new(), b.tc.error)
        };
        let gargs = generics.iter().map(|g| g.1).collect();
        let generics = generics.into_iter().collect();
        let instructions = {
            let mut var_stack = NameStack::new();
            let mut ctx = FnLowerCtx {
                instructions: Vec::new(),
                var_stack: &mut var_stack,
                b,
                output,
                origin: self.body.origin,
                generics: &generics,
                imports,
            };
            let res = self.body.lower(&mut ctx);
            let mut instructions = ctx.instructions;
            if let Some(src) = res {
                let origin = b.vars_insts[src].origin;
                instructions.push(UInstrInst {
                    origin,
                    i: UInstruction::Ret { src },
                });
            }
            instructions
        };
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
    pub generics: &'a HashMap<String, GenericID>,
}

impl<'a, 'b> FnLowerCtx<'a, 'b> {
    pub fn var(&mut self, node: &Node<parser::PIdent>) -> VarInstID {
        let inst = VarInst {
            status: if let Some(n) = node.as_ref() {
                if let Some(&var) = self.var_stack.search(&n.0) {
                    VarStatus::Var(var)
                } else {
                    VarStatus::Unres {
                        path: ModPath {
                            id: self.b.module,
                            path: Vec::new(),
                        },
                        name: n.0.clone(),
                        gargs: Vec::new(),
                        fields: Vec::new(),
                    }
                }
            } else {
                VarStatus::Cooked
            },
            origin: node.origin,
        };
        self.def_var_inst(inst)
    }
    pub fn err(&mut self, msg: String) {
        self.output.err(CompilerMsg::new(msg, self.origin))
    }
    pub fn err_at(&mut self, span: FileSpan, msg: String) {
        self.output.err(CompilerMsg::new(msg, span))
    }
    pub fn temp<T: Typable>(&mut self, ty: T) -> VarInstID {
        self.b.temp_var(self.origin, ty)
    }
    pub fn push(&mut self, i: UInstruction) {
        self.push_at(i, self.origin);
    }
    pub fn push_at(&mut self, i: UInstruction, span: FileSpan) {
        self.instructions.push(UInstrInst { i, origin: span });
    }
    pub fn branch<'c>(&'c mut self) -> FnLowerCtx<'c, 'b> {
        FnLowerCtx {
            b: self.b,
            instructions: Vec::new(),
            generics: self.generics,
            var_stack: self.var_stack,
            output: self.output,
            origin: self.origin,
            imports: self.imports,
        }
    }
}

impl<'b> Deref for FnLowerCtx<'_, 'b> {
    type Target = UModuleBuilder<'b>;

    fn deref(&self) -> &Self::Target {
        self.b
    }
}

impl DerefMut for FnLowerCtx<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.b
    }
}
