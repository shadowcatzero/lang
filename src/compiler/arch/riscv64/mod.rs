mod asm;
mod base;
mod funct;
mod opcode;
mod reg;
mod single;
mod compile;

use crate::util::BitsI32;
pub use asm::*;
use base::*;
use funct::{op::*, width};
use opcode::*;
pub use reg::*;
pub use compile::*;

use single::*;

pub fn gen() -> Vec<u8> {
    // use asm::LinkerInstruction as I;
    // let mut table = SymMap::new();
    // let (msg, len) = table.push_ro_data_size(b"Hello world!\n".to_vec());
    // let (msg2, len2) = table.push_ro_data_size(b"IT WORKS!!!!\n".to_vec());
    // let print_stuff = table.reserve();
    // let start = table.push_fn(vec![
    //     I::Call(*print_stuff),
    //     I::Li { dest: a0, imm: 0 },
    //     I::Li { dest: a7, imm: 93 },
    //     I::Ecall,
    //     I::Jal {
    //         dest: zero,
    //         offset: 0,
    //     },
    // ]);
    // table.write_fn(
    //     print_stuff,
    //     vec![
    //         I::Li { dest: a0, imm: 1 },
    //         I::La { dest: a1, src: msg },
    //         I::Li {
    //             dest: a2,
    //             imm: len as i64,
    //         },
    //         I::Li { dest: a7, imm: 64 },
    //         I::Ecall,
    //         I::Li { dest: a0, imm: 1 },
    //         I::La {
    //             dest: a1,
    //             src: msg2,
    //         },
    //         I::Li {
    //             dest: a2,
    //             imm: len2 as i64,
    //         },
    //         I::Li { dest: a7, imm: 64 },
    //         I::Ecall,
    //         I::Ret,
    //     ],
    // );
    // let (program, start) = create_program(table, Some(start));
    // elf::create(program, start.expect("no start!"))
    todo!("remove this");
}
