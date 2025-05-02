use crate::{
    ir::{GenericID, MemberID, ModPath, Type, TypeID, UGeneric, UModuleBuilder, UProgram},
    parser::PGenericDef,
};

use super::{CompilerOutput, FileSpan, Node, PType};

impl Node<Box<PType>> {
    pub fn lower(&self, p: &mut UModuleBuilder, output: &mut CompilerOutput) -> TypeID {
        self.as_ref()
            .map(|t| t.lower(p, output, self.origin))
            .unwrap_or(p.error())
    }
}
impl Node<PType> {
    pub fn lower(&self, p: &mut UModuleBuilder, output: &mut CompilerOutput) -> TypeID {
        self.as_ref()
            .map(|t| t.lower(p, output, self.origin))
            .unwrap_or(p.error())
    }
}

fn test() {}

impl PType {
    pub fn lower(
        &self,
        p: &mut UModuleBuilder,
        output: &mut CompilerOutput,
        mut origin: FileSpan,
    ) -> TypeID {
        let mut ty = self;
        let mut path = Vec::new();
        while let PType::Member(node, ident) = ty {
            ty = if let Some(t) = node.as_ref() {
                let Some(name) = ident.as_ref() else {
                    return p.error();
                };
                origin = node.origin;
                path.push(MemberID {
                    name: name.0.clone(),
                    origin: ident.origin,
                });
                &**t
            } else {
                return p.error();
            };
        }
        if !path.is_empty() {
            let PType::Ident(id) = ty else {
                return p.error();
            };
            path.push(MemberID {
                name: id.0.clone(),
                origin,
            });
            return p.def_ty(Type::Unres(ModPath { m: p.module, path }));
        }
        let ty = match ty {
            PType::Member(_, _) => unreachable!(),
            PType::Ident(node) => {
                path.push(MemberID {
                    name: node.0.clone(),
                    origin,
                });
                path.reverse();
                Type::Unres(ModPath { m: p.module, path })
            }
            PType::Ref(node) => node.lower(p, output).rf(),
            PType::Generic(node, nodes) => todo!(),
        };
        p.def_ty(ty)
        // let Some(name) = self.name.as_ref() else {
        //     return p.error();
        // };
        // let ids = p.get_idents(name);
        // // TODO: should generics always take precedence?
        // if let Some(id) = ids.and_then(|ids| ids.get::<Type>()) {
        //     Type::Generic { id }
        // } else if let Some(id) = ids.and_then(|ids| ids.get::<UStruct>()) {
        //     let args = self.args.iter().map(|n| n.lower(p, output)).collect();
        //     Type::Struct(StructInst { id, args })
        // } else if let Ok(num) = name.parse::<u32>() {
        //     Type::Bits(num)
        // } else {
        //     match name.as_str() {
        //         "slice" => {
        //             let inner = self.args[0].lower(p, output);
        //             Type::Slice(Box::new(inner))
        //         }
        //         "_" => Type::Infer,
        //         _ => {
        //             output.err(CompilerMsg::from_span(span, "Type not found".to_string()));
        //             Type::Error
        //         }
        //     }
        // }
    }
}

impl Node<PGenericDef> {
    pub fn lower(&self, p: &mut UProgram) -> Option<GenericID> {
        let s = self.as_ref()?;
        let name = s.name.as_ref()?.to_string();
        Some(p.def_generic(UGeneric {
            name,
            origin: self.origin,
        }))
    }
}
