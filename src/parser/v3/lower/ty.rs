use crate::{
    ir::{GenericID, Type, UGeneric, UProgram, UStruct},
    parser::PGenericDef,
};

use super::{CompilerMsg, CompilerOutput, FileSpan, Node, PType};

impl Node<PType> {
    pub fn lower(&self, namespace: &mut UProgram, output: &mut CompilerOutput) -> Type {
        self.as_ref()
            .map(|t| t.lower(namespace, output, self.span))
            .unwrap_or(Type::Error)
    }
}

impl PType {
    pub fn lower(&self, p: &mut UProgram, output: &mut CompilerOutput, span: FileSpan) -> Type {
        let Some(name) = self.name.as_ref() else {
            return Type::Error;
        };
        let ids = p.get_idents(name);
        // TODO: should generics always take precedence?
        if let Some(id) = ids.and_then(|ids| ids.get::<UGeneric>()) {
            Type::Generic { id }
        } else if let Some(id) = ids.and_then(|ids| ids.get::<UStruct>()) {
            let args = self.args.iter().map(|n| n.lower(p, output)).collect();
            Type::Struct { id, args }
        } else if let Ok(num) = name.parse::<u32>() {
            Type::Bits(num)
        } else {
            match name.as_str() {
                "slice" => {
                    let inner = self.args[0].lower(p, output);
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

impl Node<PGenericDef> {
    pub fn lower(&self, p: &mut UProgram) -> Option<GenericID> {
        let s = self.as_ref()?;
        let name = s.name.as_ref()?;
        Some(p.def_searchable(name.to_string(), Some(UGeneric {}), self.span))
    }
}
