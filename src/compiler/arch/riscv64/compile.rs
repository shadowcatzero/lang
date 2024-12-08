use std::collections::HashMap;

use crate::{
    compiler::{arch::riscv64::Reg, create_program, Addr},
    ir::{
        arch::riscv64::{RV64Instruction as AI, RegRef},
        IRLInstruction as IRI, IRLProgram, Len, Size, Symbol, VarID,
    },
};

use super::{LinkerInstruction as LI, *};

fn align(s: &Size) -> i32 {
    (*s as i32 - 1).div_euclid(8) + 1
}

fn mov_mem(
    v: &mut Vec<LI>,
    src: Reg,
    src_offset: i32,
    dest: Reg,
    dest_offset: i32,
    temp: Reg,
    mut len: Len,
) {
    let mut off = 0;
    while len >= 8 {
        v.extend([
            LI::Ld {
                dest: temp,
                offset: src_offset + off,
                base: src,
            },
            LI::Sd {
                src: temp,
                offset: dest_offset + off,
                base: dest,
            },
        ]);
        len -= 8;
        off += 8;
    }
}

pub fn compile(program: IRLProgram) -> (Vec<u8>, Option<Addr>) {
    let mut fns = Vec::new();
    let mut data = Vec::new();
    for (sym, d) in program.ro_data() {
        data.push((d.clone(), *sym));
    }
    for (sym, f) in program.fns() {
        let mut v = Vec::new();
        let mut stack = HashMap::new();
        let mut stack_len = 0;
        let has_stack = !f.stack.is_empty() || !f.args.is_empty() || f.makes_call;
        let mut stack_ra = None;
        if has_stack {
            if f.makes_call {
                // return addr
                stack_ra = Some(stack_len);
                stack_len += 8;
            }
            for (id, s) in &f.stack {
                stack.insert(id, stack_len);
                stack_len += align(s);
            }
            for (id, s) in f.args.iter().rev() {
                stack.insert(id, stack_len);
                stack_len += align(s);
            }
            v.push(LI::Addi {
                dest: sp,
                src: sp,
                imm: -stack_len,
            });
            if let Some(stack_ra) = stack_ra {
                v.push(LI::Sd { src: ra, offset: stack_ra, base: sp });
            }
        }
        for i in &f.instructions {
            match i {
                IRI::Mv { dest, src } => todo!(),
                IRI::Ref { dest, src } => todo!(),
                IRI::LoadAddr { dest, offset, src } => {
                    v.extend([
                        LI::La {
                            dest: t0,
                            src: *src,
                        },
                        LI::Sd {
                            src: t0,
                            offset: stack[dest] + *offset as i32,
                            base: sp,
                        },
                    ]);
                }
                IRI::LoadData {
                    dest,
                    offset,
                    src,
                    len,
                } => {
                    v.push(LI::La {
                        dest: t0,
                        src: *src,
                    });
                    mov_mem(&mut v, t0, 0, sp, stack[dest] + *offset as i32, t1, *len);
                }
                IRI::Call { dest, f, args } => {
                    let mut offset = 0;
                    for (arg, s) in args {
                        let bs = align(s);
                        offset -= bs;
                        mov_mem(&mut v, sp, stack[arg], sp, offset, t0, bs as Len);
                    }
                    v.push(LI::Call(*f));
                }
                IRI::AsmBlock { args, instructions } => {
                    for (reg, var) in args {
                        v.push(LI::Addi {
                            dest: *reg,
                            imm: stack[var],
                            src: sp,
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
        if has_stack {
            if let Some(stack_ra) = stack_ra {
                v.push(LI::Ld { dest: ra, offset: stack_ra, base: sp });
            }
            v.push(LI::Addi {
                dest: sp,
                src: sp,
                imm: stack_len,
            });
        }
        v.push(LI::Ret);
        fns.push((v, *sym));
    }
    create_program(fns, data, Some(program.entry()))
}
