mod asm_block;
mod asm_fn;
mod asm_instr;
mod block;
mod def;
mod expr;
mod func;
mod ident;
mod lit;
mod op;
mod statement;
mod struc;
mod trai;
mod ty;
mod util;

pub use asm_block::*;
pub use asm_fn::*;
pub use asm_instr::*;
pub use block::*;
pub use def::*;
pub use expr::*;
pub use func::*;
pub use ident::*;
pub use lit::*;
pub use op::*;
pub use statement::*;
pub use struc::*;
pub use trai::*;
pub use ty::*;

use crate::ir::UProgram;

use super::{lower::{FnLowerCtx, FnLowerable}, *};

pub struct PModule {
    pub block: Node<PBlock>,
}

impl PModule {
    pub fn parse(ctx: &mut ParserCtx) -> Self {
        Self {
            block: PBlock::parse_node(ctx, None).node,
        }
    }
}
