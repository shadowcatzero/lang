use crate::compiler::riscv64::AsmInstruction;

use super::{FnIdent, VarIdent};

#[derive(Debug)]
pub struct Function {
    instructions: Vec<Instruction>,
}

#[derive(Debug)]
pub enum Instruction {
    Mv {
        dest: VarIdent,
        src: VarIdent,
    },
    Ref {
        dest: VarIdent,
        src: VarIdent,
    },
    Lf {
        dest: VarIdent,
        src: FnIdent,
    },
    Call {
        dest: VarIdent,
        f: FnIdent,
        args: Vec<VarIdent>,
    },
    AsmBlock {
        instructions: Vec<AsmInstruction>,
    },
    Ret {
        src: VarIdent,
    },
}

pub struct Instructions {
    vec: Vec<Instruction>,
}

impl Function {
    pub fn new(instructions: Instructions) -> Self {
        Self {
            instructions: instructions.vec,
        }
    }
}

impl Instructions {
    pub fn new() -> Self {
        Self { vec: Vec::new() }
    }
    pub fn push(&mut self, i: Instruction) {
        self.vec.push(i);
    }
}
