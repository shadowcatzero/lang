pub mod arch;
mod debug;
mod elf;
mod program;
mod target;

use arch::riscv;
pub use program::*;

use crate::ir::LProgram;

pub fn compile(program: &LProgram) -> UnlinkedProgram<riscv::LinkerInstruction> {
    arch::riscv::compile(program)
}
