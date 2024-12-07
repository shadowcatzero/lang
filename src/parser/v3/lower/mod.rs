mod asm;
mod block;
mod def;
mod expr;
mod func;
mod module;
mod arch;

use super::*;

pub use func::FnLowerCtx;

pub trait FnLowerable {
    type Output;
    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<Self::Output>;
}

impl<T: FnLowerable> FnLowerable for Node<T> {
    type Output = T::Output;
    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<T::Output> {
        self.as_ref()?.lower(&mut ctx.span(self.span))
    }
}

impl<T: FnLowerable> FnLowerable for Box<T> {
    type Output = T::Output;
    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<T::Output> {
        self.as_ref().lower(ctx)
    }
}
