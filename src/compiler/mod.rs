pub mod arch;
mod elf;
mod program;
mod target;

pub use program::*;

use crate::ir::IRLProgram;

pub fn compile(program: IRLProgram) -> Vec<u8> {
    let (compiled, start) = arch::riscv64::compile(program);
    elf::create(compiled, start.expect("no start method found"))
}
