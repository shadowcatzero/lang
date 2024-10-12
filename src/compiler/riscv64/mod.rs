mod asm;
mod base;
mod funct;
mod opcode;
mod reg;
mod single;

use std::collections::HashMap;

use super::{elf, program::{create_program, Function}};
use crate::util::BitsI32;
use base::*;
use funct::{op::*, width};
use opcode::*;
pub use reg::*;

use single::*;

pub fn gen() -> Vec<u8> {
    use asm::AsmInstruction as I;
    // let mut program = Vec::new();
    // let msg = b"Hello world!\n";
    // program.extend(msg);
    // program.resize(((program.len() - 1) / 4 + 1) * 4, 0);
    // let start = program.len() as u64;
    // let instructions = [
    //     auipc(t0, BitsI32::new(0)),
    //     addi(t0, t0, BitsI32::new(-(start as i32))),
    //     addi(a0, zero, BitsI32::new(1)),
    //     mv(a1, t0),
    //     addi(a2, zero, BitsI32::new(msg.len() as i32)),
    //     addi(a7, zero, BitsI32::new(64)),
    //     addi(t0, zero, const { BitsI32::new(-10) }),
    //     ecall(),
    //     // exit
    //     addi(a0, zero, BitsI32::new(0)),
    //     addi(a7, zero, BitsI32::new(93)),
    //     ecall(),
    //     j(BitsI32::new(0)),
    // ];
    // for i in instructions {
    //     program.extend(i.to_le_bytes());
    // }
    let msg = b"Hello world!\n";
    let msg2 = b"IT WORKS!!!!\n";
    let ro_data = HashMap::from([
        ("msg".to_string(), msg.to_vec()),
        ("msg2".to_string(), msg2.to_vec()),
    ]);
    let functions = vec![
        Function {
            label: "print_stuff".to_string(),
            instructions: vec![
                I::Addi(a0, zero, 1),
                I::La(a1, "msg".to_string()),
                I::Addi(a2, zero, msg.len() as i32),
                I::Addi(a7, zero, 64),
                I::Ecall,
                I::Addi(a0, zero, 1),
                I::La(a1, "msg2".to_string()),
                I::Addi(a2, zero, msg2.len() as i32),
                I::Addi(a7, zero, 64),
                I::Ecall,
                I::Ret,
            ]
        },
        Function {
            label: "_start".to_string(),
            instructions: vec![
                I::Jala("print_stuff".to_string()),
                I::Ecall,
                I::Addi(a0, zero, 0),
                I::Addi(a7, zero, 93),
                I::Ecall,
                I::Jal(zero, 0),
            ]
        },
    ];
    let (program, start) = create_program(ro_data, functions);
    elf::create(program, start.expect("no start!"))
}
