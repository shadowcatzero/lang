use crate::{
    ir::{GenericID, MemberIdent, MemberPath, Type, TypeID, UGeneric, UProgram},
    parser::PGenericDef,
};

use super::{FileSpan, ModuleLowerCtx, Node, PType};

impl Node<Box<PType>> {
    pub fn lower(&self, ctx: &mut ModuleLowerCtx) -> TypeID {
        self.as_ref()
            .map(|t| t.lower(ctx, self.origin))
            .unwrap_or(ctx.tc.error)
    }
}
impl Node<PType> {
    pub fn lower(&self, ctx: &mut ModuleLowerCtx) -> TypeID {
        self.as_ref()
            .map(|t| t.lower(ctx, self.origin))
            .unwrap_or(ctx.tc.error)
    }
}

impl PType {
    pub fn lower(&self, ctx: &mut ModuleLowerCtx, mut origin: FileSpan) -> TypeID {
        let mut ty = self;
        let mut path = Vec::new();
        while let PType::Member(node, ident) = ty {
            ty = if let Some(t) = node.as_ref() {
                let Some(name) = ident.as_ref() else {
                    return ctx.tc.error;
                };
                origin = node.origin;
                path.push(MemberIdent {
                    name: name.0.clone(),
                    origin: ident.origin,
                });
                &**t
            } else {
                return ctx.tc.error;
            };
        }
        if !path.is_empty() {
            let PType::Ident(id) = ty else {
                return ctx.tc.error;
            };
            path.push(MemberIdent {
                name: id.0.clone(),
                origin,
            });
            path.reverse();
            let ty = Type::Unres(MemberPath {
                id: ctx.module,
                path,
            });
            return ctx.def_ty(ty);
        }
        let ty = match ty {
            PType::Member(_, _) => unreachable!(),
            PType::Ident(node) => {
                path.push(MemberIdent {
                    name: node.0.clone(),
                    origin,
                });
                path.reverse();
                Type::Unres(MemberPath {
                    id: ctx.module,
                    path,
                })
            }
            PType::Ref(node) => node.lower(ctx).rf(),
            PType::Generic(node, nodes) => todo!(),
        };
        ctx.def_ty(ty)
    }
}

impl Node<PGenericDef> {
    pub fn lower(&self, p: &mut UProgram) -> Option<(String, GenericID)> {
        let s = self.as_ref()?;
        let name = s.name.as_ref()?.to_string();
        Some((
            name.clone(),
            p.def_generic(UGeneric {
                name,
                origin: self.origin,
            }),
        ))
    }
}
