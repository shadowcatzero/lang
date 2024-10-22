use super::{Function as PFunction, Node, ParserMsg, ParserOutput};
use crate::ir::{
    BuiltinType, FileSpan, FnDef, FnIdent, Function, Idents, Instruction, Instructions,
    NamespaceGuard, Origin, Type, VarDef, VarIdent,
};

impl Node<PFunction> {
    pub fn lower_header(
        &self,
        map: &mut NamespaceGuard,
        output: &mut ParserOutput,
    ) -> Option<FnIdent> {
        self.as_ref()?.lower_header(map, output)
    }
    pub fn lower_body(&self, map: &mut NamespaceGuard, output: &mut ParserOutput) -> Option<Function> {
        if let Some(f) = self.as_ref() {
            Some(f.lower_body(map, output))
        } else {
            None
        }
    }
}

impl PFunction {
    pub fn lower_header(
        &self,
        map: &mut NamespaceGuard,
        output: &mut ParserOutput,
    ) -> Option<FnIdent> {
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
            None => Type::Concrete(BuiltinType::Unit.id()),
        };
        // ignoring self var for now
        Some(map.def_fn(FnDef {
            name: name.val().clone(),
            origin: Origin::File(self.header.span),
            args,
            ret,
        }))
    }
    pub fn lower_body(&self, map: &mut NamespaceGuard, output: &mut ParserOutput) -> Function {
        let mut instructions = Instructions::new();
        let mut ctx = FnLowerCtx {
            instructions: &mut instructions,
            map,
            output,
            span: self.body.span,
        };
        if let Some(res) = self.body.lower(&mut ctx) {
            match res {
                super::ExprResult::Var(v) => instructions.push(Instruction::Ret { src: v }),
                super::ExprResult::Fn(_) => todo!(),
            }
        }
        Function::new(instructions)
    }
}

pub struct FnLowerCtx<'a, 'n> {
    pub map: &'a mut NamespaceGuard<'n>,
    pub instructions: &'a mut Instructions,
    pub output: &'a mut ParserOutput,
    pub span: FileSpan,
}

impl<'a, 'n> FnLowerCtx<'a, 'n> {
    pub fn get(&self, name: &str) -> Option<Idents> {
        self.map.get(name)
    }
    pub fn err(&mut self, msg: String) {
        self.output.err(ParserMsg::from_span(self.span, msg))
    }
    pub fn temp(&mut self, ty: Type) -> VarIdent {
        self.map.temp_var(self.span, ty)
    }
    pub fn push(&mut self, i: Instruction) {
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
