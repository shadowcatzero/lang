use crate::{
    ir::{StructInst, Type, TypeID, UGeneric, UProgram, UStruct},
    parser::PGenericDef,
};

use super::{CompilerMsg, CompilerOutput, FileSpan, Node, PType};

impl Node<PType> {
    pub fn lower(&self, namespace: &mut UProgram, output: &mut CompilerOutput) -> TypeID {
        self.as_ref()
            .map(|t| t.lower(namespace, output, self.origin))
            .unwrap_or(Type::Error)
    }
}

impl PType {
    pub fn lower(&self, p: &mut UProgram, output: &mut CompilerOutput, span: FileSpan) -> TypeID {
        let Some(name) = self.name.as_ref() else {
            return p.error_type();
        };
        let ids = p.get_idents(name);
        // TODO: should generics always take precedence?
        if let Some(id) = ids.and_then(|ids| ids.get::<Type>()) {
            Type::Generic { id }
        } else if let Some(id) = ids.and_then(|ids| ids.get::<UStruct>()) {
            let args = self.args.iter().map(|n| n.lower(p, output)).collect();
            Type::Struct(StructInst { id, args })
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
        Some(p.def_searchable(name, Some(UGeneric {}), self.origin))
    }
}
