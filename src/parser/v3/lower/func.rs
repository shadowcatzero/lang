use super::{CompilerMsg, CompilerOutput, FileSpan, FnLowerable, Node, PFunction};
use crate::{
    ir::{
        FnDef, FnID, IRUFunction, IRUInstrInst, IRUInstruction, IRUProgram, Idents, Origin, Type,
        VarDef, VarInst,
    },
    parser,
};

impl Node<PFunction> {
    pub fn lower_header(&self, map: &mut IRUProgram, output: &mut CompilerOutput) -> Option<FnID> {
        self.as_ref()?.lower_header(map, output)
    }
    pub fn lower_body(
        &self,
        id: FnID,
        map: &mut IRUProgram,
        output: &mut CompilerOutput,
    ) -> Option<IRUFunction> {
        Some(self.as_ref()?.lower_body(id, map, output))
    }
}

impl PFunction {
    pub fn lower_header(&self, map: &mut IRUProgram, output: &mut CompilerOutput) -> Option<FnID> {
        let header = self.header.as_ref()?;
        let name = header.name.as_ref()?;
        let args = header
            .args
            .iter()
            .map(|a| {
                a.lower(map, output).unwrap_or(VarDef {
                    name: "{error}".to_string(),
                    origin: Origin::File(a.span),
                    ty: Type::Error,
                })
            })
            .collect();
        let ret = match &header.ret {
            Some(ty) => ty.lower(map, output),
            None => Type::Unit,
        };
        Some(map.def_fn(FnDef {
            name: name.to_string(),
            origin: Origin::File(self.header.span),
            args,
            ret,
        }))
    }
    pub fn lower_body(
        &self,
        id: FnID,
        map: &mut IRUProgram,
        output: &mut CompilerOutput,
    ) -> IRUFunction {
        let def = map.get_fn(id).clone();
        let args = def.args.iter().map(|a| map.named_var(a.clone())).collect();
        let mut ctx = FnLowerCtx {
            instructions: Vec::new(),
            program: map,
            output,
            span: self.body.span,
        };
        if let Some(src) = self.body.lower(&mut ctx) {
            ctx.instructions.push(IRUInstrInst {
                i: IRUInstruction::Ret { src },
                span: src.span,
            });
        }
        IRUFunction {
            name: def.name.clone(),
            args,
            ret: def.ret,
            instructions: ctx.instructions,
        }
    }
}

pub struct FnLowerCtx<'a> {
    pub program: &'a mut IRUProgram,
    pub instructions: Vec<IRUInstrInst>,
    pub output: &'a mut CompilerOutput,
    pub span: FileSpan,
}

impl FnLowerCtx<'_> {
    pub fn get(&mut self, node: &Node<parser::PIdent>) -> Option<Idents> {
        let name = node.inner.as_ref()?;
        let res = self.program.get(name);
        if res.is_none() {
            self.err_at(node.span, format!("Identifier '{}' not found", name));
        }
        res
    }
    pub fn get_var(&mut self, node: &Node<parser::PIdent>) -> Option<VarInst> {
        let ids = self.get(node)?;
        if ids.var.is_none() {
            self.err_at(
                node.span,
                format!(
                    "Variable '{}' not found; Type found but cannot be used here",
                    node.inner.as_ref()?
                ),
            );
        }
        ids.var.map(|id| VarInst {
            id,
            span: node.span,
        })
    }
    pub fn err(&mut self, msg: String) {
        self.output.err(CompilerMsg::from_span(self.span, msg))
    }
    pub fn err_at(&mut self, span: FileSpan, msg: String) {
        self.output.err(CompilerMsg::from_span(span, msg))
    }
    pub fn temp(&mut self, ty: Type) -> VarInst {
        self.program.temp_var(self.span, ty)
    }
    pub fn push(&mut self, i: IRUInstruction) {
        self.instructions.push(IRUInstrInst { i, span: self.span });
    }
    pub fn push_at(&mut self, i: IRUInstruction, span: FileSpan) {
        self.instructions.push(IRUInstrInst { i, span });
    }
    pub fn branch<'a>(&'a mut self) -> FnLowerCtx<'a> {
        FnLowerCtx {
            program: self.program,
            instructions: Vec::new(),
            output: self.output,
            span: self.span,
        }
    }
}
