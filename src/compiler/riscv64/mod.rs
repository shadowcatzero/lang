mod asm;
mod base;
mod funct;
mod opcode;
mod reg;
mod single;

use super::{create_program, elf, SymMap};
use crate::util::BitsI32;
use base::*;
use funct::{op::*, width};
use opcode::*;
pub use reg::*;

use single::*;

pub fn gen() -> Vec<u8> {
    use asm::AsmInstruction as I;
    let mut table = SymMap::new();
    let (msg, len) = table.push_ro_data(b"Hello world!\n");
    let (msg2, len2) = table.push_ro_data(b"IT WORKS!!!!\n");
    let print_stuff = table.reserve();
    let start = table.push_fn(vec![
        I::Call(*print_stuff),
        I::Li(a0, 0),
        I::Li(a7, 93),
        I::Ecall,
        I::Jal(zero, 0),
    ]);
    table.write_fn(
        print_stuff,
        vec![
            I::Li(a0, 1),
            I::La(a1, msg),
            I::Li(a2, len as i32),
            I::Li(a7, 64),
            I::Ecall,
            I::Li(a0, 1),
            I::La(a1, msg2),
            I::Li(a2, len2 as i32),
            I::Li(a7, 64),
            I::Ecall,
            I::Ret,
        ],
    );
    let (program, start) = create_program(table, start);
    elf::create(program, start.expect("no start!"))
}
