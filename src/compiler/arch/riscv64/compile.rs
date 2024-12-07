use std::collections::HashMap;

use crate::{
    compiler::{arch::riscv64::Reg, create_program, Addr},
    ir::{
        arch::riscv64::{RV64Instruction as AI, RegRef},
        IRLInstruction as IRI, Program,
    },
};

use super::{LinkerInstruction as LI, *};

pub fn compile(program: Program) -> (Vec<u8>, Option<Addr>) {
    let mut fns = Vec::new();
    let mut data = Vec::new();
    for d in program.data {
        data.push((d.data, d.addr));
    }
    let mut start = None;
    for f in program.fns {
        let mut v = Vec::new();
        let mut stack = HashMap::new();
        let mut stack_len = 0;
        if !f.stack.is_empty() || !f.args.is_empty() {
            for (id, s) in &f.stack {
                stack.insert(id, stack_len);
                stack_len += *s as i32;
            }
            for (id, s) in f.args.iter().rev() {
                stack.insert(id, stack_len);
                stack_len += *s as i32;
            }
            v.push(LI::Addi {
                dest: sp,
                src: sp,
                imm: -stack_len,
            });
        }
        for i in &f.instructions {
            match i {
                IRI::Mv { dest, src } => todo!(),
                IRI::Ref { dest, src } => todo!(),
                IRI::LoadAddr { dest, src } => {
                    v.extend([
                        LI::La {
                            dest: t0,
                            src: *src,
                        },
                        LI::Sd {
                            src: t0,
                            offset: stack[dest],
                            base: sp,
                        },
                    ]);
                }
                IRI::Call { dest, f, args } => {
                    let mut offset = 0;
                    for (arg, s) in args {
                        offset -= *s as i32;
                        v.extend([
                            LI::Ld {
                                dest: t0,
                                offset: stack[arg],
                                base: sp,
                            },
                            LI::Sd {
                                src: t0,
                                offset,
                                base: sp,
                            },
                        ]);
                    }
                    v.push(LI::Call(*f));
                }
                IRI::AsmBlock { args, instructions } => {
                    for (reg, var) in args {
                        v.push(LI::Ld {
                            dest: *reg,
                            offset: stack[var],
                            base: sp,
                        });
                    }
                    fn r(rr: RegRef) -> Reg {
                        match rr {
                            RegRef::Var(var_ident) => todo!(),
                            RegRef::Reg(reg) => reg,
                        }
                    }
                    for i in instructions {
                        match *i {
                            AI::Ecall => v.push(LI::Ecall),
                            AI::Li { dest, imm } => v.push(LI::Li { dest: r(dest), imm }),
                            AI::Mv { dest, src } => v.push(LI::Mv {
                                dest: r(dest),
                                src: r(src),
                            }),
                            AI::La { dest, src } => todo!(),
                            AI::Ld { dest, base, offset } => v.push(LI::Ld {
                                dest: r(dest),
                                offset: offset as i32,
                                base: r(base),
                            }),
                        }
                    }
                }
                IRI::Ret { src } => todo!(),
            }
        }
        if f.name == "start" {
            start = Some(f.addr);
        } else {
            v.push(LI::Ret);
        }
        fns.push((v, f.addr));
    }
    create_program(fns, data, start)
}
