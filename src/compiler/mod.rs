pub mod arch;
mod debug;
mod elf;
mod program;
mod target;

use arch::riscv;
pub use program::*;

use crate::ir::IRLProgram;

pub fn compile(program: &IRLProgram) -> UnlinkedProgram<riscv::LinkerInstruction> {
    arch::riscv::compile(program)
}
