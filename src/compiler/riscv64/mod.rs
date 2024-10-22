mod asm;
mod base;
mod funct;
mod opcode;
mod parse;
mod reg;
mod single;

use super::{create_program, elf, SymMap};
use crate::util::BitsI32;
pub use asm::*;
use base::*;
use funct::{op::*, width};
use opcode::*;
pub use parse::*;
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
        I::Li { dest: a0, imm: 0 },
        I::Li { dest: a7, imm: 93 },
        I::Ecall,
        I::Jal {
            dest: zero,
            offset: 0,
        },
    ]);
    table.write_fn(
        print_stuff,
        vec![
            I::Li { dest: a0, imm: 1 },
            I::La { dest: a1, sym: msg },
            I::Li {
                dest: a2,
                imm: len as i64,
            },
            I::Li { dest: a7, imm: 64 },
            I::Ecall,
            I::Li { dest: a0, imm: 1 },
            I::La {
                dest: a1,
                sym: msg2,
            },
            I::Li {
                dest: a2,
                imm: len2 as i64,
            },
            I::Li { dest: a7, imm: 64 },
            I::Ecall,
            I::Ret,
        ],
    );
    let (program, start) = create_program(table, start);
    elf::create(program, start.expect("no start!"))
}
