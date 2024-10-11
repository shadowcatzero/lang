use crate::compiler::program::Instr;
mod elf;
mod instruction;
mod asm;

use instruction::*;

pub fn gen() -> Vec<u8> {
    let mut program = Vec::new();
    let msg = b"Hello world!\n";
    program.extend(msg);
    program.resize(((program.len() - 1) / 4 + 1) * 4, 0);
    let start = program.len() as u64;
    let instructions = [
        auipc(t0, 0),
        addi(t0, t0, -(start as i32)),
        addi(a0, zero, 1),
        mv(a1, t0),
        addi(a2, zero, msg.len() as i32),
        addi(a7, zero, 64),
        addi(t0, zero, 200),
        ecall(),
        // exit
        addi(a0, zero, 0),
        addi(a7, zero, 93),
        ecall(),
        j(0),
    ];
    for i in instructions {
        program.extend(i.to_le_bytes());
    }
    elf::create(program, start)
}
