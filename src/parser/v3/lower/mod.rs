mod arch;
mod asm;
mod block;
mod def;
mod expr;
mod func;
mod module;
mod struc;

use super::*;

pub use func::FnLowerCtx;

pub trait FnLowerable {
    type Output;
    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<Self::Output>;
}

impl<T: FnLowerable> FnLowerable for Node<T> {
    type Output = T::Output;
    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<T::Output> {
        let old_span = ctx.span;
        ctx.span = self.span;
        let res = self.as_ref()?.lower(ctx);
        ctx.span = old_span;
        res
    }
}

impl<T: FnLowerable> FnLowerable for Box<T> {
    type Output = T::Output;
    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<T::Output> {
        self.as_ref().lower(ctx)
    }
}
