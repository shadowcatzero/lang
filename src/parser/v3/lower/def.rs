use crate::ir::{FileSpan, NamespaceGuard, Origin, Type, VarDef};

use super::{Node, ParserMsg, ParserOutput, PType, PVarDef};

impl Node<PVarDef> {
    pub fn lower(
        &self,
        namespace: &mut NamespaceGuard,
        output: &mut ParserOutput,
    ) -> Option<VarDef> {
        let s = self.as_ref()?;
        let name = s.name.as_ref()?.to_string();
        let ty = match &s.ty {
            Some(ty) => ty.lower(namespace, output),
            None => Type::Infer,
        };
        Some(VarDef {
            name,
            ty,
            origin: Origin::File(self.span),
        })
    }
}

impl Node<PType> {
    pub fn lower(&self, namespace: &mut NamespaceGuard, output: &mut ParserOutput) -> Type {
        self.as_ref()
            .map(|t| t.lower(namespace, output, self.span))
            .unwrap_or(Type::Error)
    }
}

impl PType {
    pub fn lower(
        &self,
        namespace: &mut NamespaceGuard,
        output: &mut ParserOutput,
        span: FileSpan,
    ) -> Type {
        match namespace.get(&self.name).and_then(|ids| ids.ty) {
            Some(id) => {
                if self.args.is_empty() {
                    Type::Concrete(id)
                } else {
                    let args = self
                        .args
                        .iter()
                        .map(|n| n.lower(namespace, output))
                        .collect();
                    Type::Generic { base: id, args }
                }
            }
            None => {
                if let Ok(num) = self.name.parse::<u32>() {
                    Type::Bits(num)
                } else {
                    output.err(ParserMsg::from_span(span, "Type not found".to_string()));
                    Type::Error
                }
            }
        }
    }
}
