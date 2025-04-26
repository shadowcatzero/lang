mod arch;
mod asm;
mod block;
mod def;
mod expr;
mod func;
mod struc;
mod ty;

use super::*;
use crate::ir::{Type, UFunc, UProgram};

impl PModule {
    pub fn lower(
        &self,
        name: String,
        p: &mut UProgram,
        imports: &mut Imports,
        output: &mut CompilerOutput,
    ) {
        let fid = p.def_searchable(name.clone(), None, self.block.origin);
        p.push_name(&name);
        let mut fctx = FnLowerCtx {
            program: p,
            instructions: Vec::new(),
            output,
            origin: self.block.origin,
            imports,
        };
        self.block.lower(&mut fctx);
        let f = UFunc {
            args: Vec::new(),
            instructions: fctx.instructions,
            ret: Type::Unit,
        };
        p.write(fid, f);
        p.pop_name();
    }
}

pub use func::FnLowerCtx;
use import::Imports;

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
