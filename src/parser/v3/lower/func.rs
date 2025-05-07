use std::ops::{Deref, DerefMut};

use super::{CompilerMsg, FileSpan, ModuleLowerCtx, Node, PFunction, Typable};
use crate::{
    ir::{
        FnID, IdentID, IdentStatus, MemRes, Member, MemberID, MemberIdent, MemberPath, MemberTy,
        Origin, Res, Type, UFunc, UIdent, UInstrInst, UInstruction,
    },
    parser,
};

impl Node<PFunction> {
    pub fn lower(&self, ctx: &mut ModuleLowerCtx) -> Option<FnID> {
        self.as_ref().map(|s| s.lower(ctx, self.origin)).flatten()
    }
}

impl PFunction {
    pub fn lower(&self, ctx: &mut ModuleLowerCtx, origin: Origin) -> Option<FnID> {
        let header = self.header.as_ref()?;
        let name = header.name.as_ref()?.0.clone();
        let (generics, args, ret) = if let Some(header) = self.header.as_ref() {
            (
                header
                    .gargs
                    .iter()
                    .flat_map(|a| a.lower(ctx).map(|g| (g.0, g.1, a.origin)))
                    .collect(),
                header
                    .args
                    .iter()
                    .flat_map(|a| Some(a.lower(ctx)?))
                    .collect(),
                match &header.ret {
                    Some(ty) => ty.lower(ctx),
                    None => ctx.def_ty(Type::Unit),
                },
            )
        } else {
            (Vec::new(), Vec::new(), ctx.tc.error)
        };
        let gargs = generics.iter().map(|g| g.1).collect();
        let generics = generics
            .into_iter()
            .map(|g| {
                (
                    g.0,
                    ctx.def_ident(UIdent {
                        status: IdentStatus::Res(Res::Generic(g.1)),
                        origin: g.2,
                    }),
                )
            })
            .collect::<Vec<_>>();
        ctx.ident_stack.extend(generics.into_iter());
        let instructions = {
            let mut fctx = FnLowerCtx {
                instructions: Vec::new(),
                ctx,
                origin: self.body.origin,
            };
            let res = self.body.lower(&mut fctx);
            let mut instructions = fctx.instructions;
            if let Some(src) = res {
                let origin = ctx.idents[src].origin;
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
        Some(ctx.def_fn(f))
    }
}

pub struct FnLowerCtx<'a, 'b> {
    pub ctx: &'a mut ModuleLowerCtx<'b>,
    pub instructions: Vec<UInstrInst>,
    pub origin: FileSpan,
}

impl<'a, 'b> FnLowerCtx<'a, 'b> {
    pub fn ident(&mut self, node: &Node<parser::PIdent>) -> IdentID {
        let inst = UIdent {
            status: if let Some(n) = node.as_ref() {
                if let Some(&res) = self.ident_stack.search(&n.0) {
                    return res;
                } else {
                    IdentStatus::Unres {
                        path: vec![MemberIdent {
                            ty: MemberTy::Member,
                            name: n.0.clone(),
                            origin: node.origin,
                            gargs: Vec::new(),
                        }],
                        base: MemRes {
                            mem: Member {
                                id: MemberID::Module(self.module),
                            },
                            origin: self.origin,
                            gargs: Vec::new(),
                        },
                    }
                }
            } else {
                IdentStatus::Cooked
            },
            origin: node.origin,
        };
        self.def_ident(inst)
    }
    pub fn err(&mut self, msg: String) {
        let origin = self.origin;
        self.output.err(CompilerMsg::new(msg, origin))
    }
    pub fn err_at(&mut self, span: FileSpan, msg: String) {
        self.output.err(CompilerMsg::new(msg, span))
    }
    pub fn temp<T: Typable>(&mut self, ty: T) -> IdentID {
        self.ctx.temp_var(self.origin, ty)
    }
    pub fn push(&mut self, i: UInstruction) {
        self.push_at(i, self.origin);
    }
    pub fn push_at(&mut self, i: UInstruction, span: FileSpan) {
        self.instructions.push(UInstrInst { i, origin: span });
    }
    pub fn branch<'c>(&'c mut self) -> FnLowerCtx<'c, 'b> {
        FnLowerCtx {
            ctx: self.ctx,
            instructions: Vec::new(),
            origin: self.origin,
        }
    }
}

impl<'b> Deref for FnLowerCtx<'_, 'b> {
    type Target = ModuleLowerCtx<'b>;

    fn deref(&self) -> &Self::Target {
        self.ctx
    }
}

impl DerefMut for FnLowerCtx<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ctx
    }
}

pub trait FnLowerable {
    type Output;
    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<Self::Output>;
}

impl<T: FnLowerable> FnLowerable for Node<T> {
    type Output = T::Output;
    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<T::Output> {
        let old_span = ctx.origin;
        ctx.origin = self.origin;
        let res = self.as_ref()?.lower(ctx);
        ctx.origin = old_span;
        res
    }
}

impl<T: FnLowerable> FnLowerable for Box<T> {
    type Output = T::Output;
    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<T::Output> {
        self.as_ref().lower(ctx)
    }
}
