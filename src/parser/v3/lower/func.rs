use super::{FnLowerable, Node, PFunction, ParserMsg, ParserOutput};
use crate::{
    ir::{
        FileSpan, FnDef, FnID, IRInstructions, IRUFunction, IRUInstruction, Idents, NamespaceGuard,
        Origin, Type, VarDef, VarID,
    },
    parser,
};

impl Node<PFunction> {
    pub fn lower_header(
        &self,
        map: &mut NamespaceGuard,
        output: &mut ParserOutput,
    ) -> Option<FnID> {
        self.as_ref()?.lower_header(map, output)
    }
    pub fn lower_body(
        &self,
        id: FnID,
        map: &mut NamespaceGuard,
        output: &mut ParserOutput,
    ) -> Option<IRUFunction> {
        Some(self.as_ref()?.lower_body(id, map, output))
    }
}

impl PFunction {
    pub fn lower_header(
        &self,
        map: &mut NamespaceGuard,
        output: &mut ParserOutput,
    ) -> Option<FnID> {
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
        map: &mut NamespaceGuard,
        output: &mut ParserOutput,
    ) -> IRUFunction {
        let mut instructions = IRInstructions::new();
        let def = map.get_fn(id).clone();
        let args = def.args.iter().map(|a| map.named_var(a.clone())).collect();
        let mut ctx = FnLowerCtx {
            instructions: &mut instructions,
            map,
            output,
            span: self.body.span,
        };
        if let Some(src) = self.body.lower(&mut ctx) {
            instructions.push(IRUInstruction::Ret { src });
        }
        IRUFunction::new(def.name.clone(), args, instructions)
    }
}

pub struct FnLowerCtx<'a, 'n> {
    pub map: &'a mut NamespaceGuard<'n>,
    pub instructions: &'a mut IRInstructions,
    pub output: &'a mut ParserOutput,
    pub span: FileSpan,
}

impl<'n> FnLowerCtx<'_, 'n> {
    pub fn span<'b>(&'b mut self, span: FileSpan) -> FnLowerCtx<'b, 'n> {
        FnLowerCtx {
            map: self.map,
            instructions: self.instructions,
            output: self.output,
            span,
        }
    }
    pub fn get(&mut self, node: &Node<parser::PIdent>) -> Option<Idents> {
        let name = node.inner.as_ref()?;
        let res = self.map.get(name);
        if res.is_none() {
            self.err_at(node.span, format!("Identifier '{}' not found", name));
        }
        res
    }
    pub fn get_var(&mut self, node: &Node<parser::PIdent>) -> Option<VarID> {
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
        ids.var
    }
    pub fn err(&mut self, msg: String) {
        self.output.err(ParserMsg::from_span(self.span, msg))
    }
    pub fn err_at(&mut self, span: FileSpan, msg: String) {
        self.output.err(ParserMsg::from_span(span, msg))
    }
    pub fn temp(&mut self, ty: Type) -> VarID {
        self.map.temp_var(self.span, ty)
    }
    pub fn push(&mut self, i: IRUInstruction) {
        self.instructions.push(i);
    }
    pub fn sub<'b>(&'b mut self) -> FnLowerCtx<'b, 'n> {
        FnLowerCtx {
            map: self.map,
            instructions: self.instructions,
            output: self.output,
            span: self.span,
        }
    }
}
