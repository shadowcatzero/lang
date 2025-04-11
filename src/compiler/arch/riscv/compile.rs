use std::collections::HashMap;

use crate::{
    compiler::{arch::riscv::Reg, debug::DebugInfo, UnlinkedFunction, UnlinkedProgram},
    ir::{
        arch::riscv64::{RV64Instruction as AI, RegRef},
        LInstruction as IRI, LProgram, Len, Size,
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
    for width in width::MAIN.iter().rev().copied() {
        let wl = width::len(width);
        while len >= wl {
            v.extend([
                LI::Load {
                    width,
                    dest: temp,
                    offset: src_offset + off,
                    base: src,
                },
                LI::Store {
                    width,
                    src: temp,
                    offset: dest_offset + off,
                    base: dest,
                },
            ]);
            len -= wl;
            off += wl as i32;
        }
    }
}

pub fn compile(program: &LProgram) -> UnlinkedProgram<LI> {
    let mut fns = Vec::new();
    let mut data = Vec::new();
    let mut dbg = DebugInfo::new(program.labels().to_vec());
    for (sym, d) in program.ro_data() {
        data.push((d.clone(), *sym));
    }
    for (sym, f) in program.fns() {
        let mut v = Vec::new();
        let mut stack = HashMap::new();
        let mut stack_len = 0;
        let mut stack_ra = None;
        let mut stack_rva = None;
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
        if f.ret_size > 0 {
            stack_rva = Some(stack_len);
            stack_len += align(&f.ret_size);
        }
        v.push(LI::addi(sp, sp, -stack_len));
        for (id, var) in &f.subvar_map {
            // TODO: ALIGN DOES NOT MAKE SENSE HERE!!! need to choose to decide in lower or asm
            stack.insert(id, stack[&var.id] + align(&var.offset));
        }
        let has_stack = stack_len > 0;
        if has_stack {
            if let Some(stack_ra) = stack_ra {
                v.push(LI::sd(ra, stack_ra, sp));
            }
        }
        let mut locations = HashMap::new();
        let mut irli = Vec::new();
        let mut ret = Vec::new();
        if has_stack {
            if let Some(stack_ra) = stack_ra {
                ret.push(LI::ld(ra, stack_ra, sp));
            }
            ret.push(LI::addi(sp, sp, stack_len));
        }
        ret.push(LI::Ret);

        for i in &f.instructions {
            irli.push((v.len(), format!("{i:?}")));
            match i {
                IRI::Mv {
                    dest,
                    dest_offset,
                    src,
                    src_offset,
                } => {
                    let s = align(&f.stack[src]) as u32;
                    mov_mem(
                        &mut v,
                        sp,
                        stack[src] + align(src_offset),
                        sp,
                        stack[dest] + align(dest_offset),
                        t0,
                        s,
                    );
                }
                IRI::Ref { dest, src } => {
                    v.push(LI::addi(t0, sp, stack[src]));
                    v.push(LI::sd(t0, stack[dest], sp));
                }
                IRI::LoadAddr { dest, offset, src } => {
                    v.extend([
                        LI::La {
                            dest: t0,
                            src: *src,
                        },
                        LI::sd(t0, stack[dest] + *offset as i32, sp),
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
                    if let Some((dest, s)) = dest {
                        offset -= align(s);
                        v.push(LI::addi(t0, sp, stack[&dest]));
                        v.push(LI::sd(t0, offset, sp))
                    }
                    for (arg, s) in args {
                        let bs = align(s);
                        offset -= bs;
                        mov_mem(&mut v, sp, stack[arg], sp, offset, t0, bs as Len);
                    }
                    v.push(LI::Call(*f));
                }
                IRI::AsmBlock {
                    inputs,
                    outputs,
                    instructions,
                } => {
                    for (reg, var) in inputs {
                        v.push(LI::ld(*reg, stack[var], sp));
                    }
                    fn r(rr: RegRef) -> Reg {
                        match rr {
                            RegRef::Var(..) => todo!(),
                            RegRef::Reg(reg) => reg,
                        }
                    }
                    for i in instructions {
                        match *i {
                            AI::ECall => v.push(LI::ECall),
                            AI::EBreak => v.push(LI::EBreak),
                            AI::Li { dest, imm } => v.push(LI::Li { dest: r(dest), imm }),
                            AI::Mv { dest, src } => v.push(LI::Mv {
                                dest: r(dest),
                                src: r(src),
                            }),
                            AI::La { .. } => todo!(),
                            AI::Load {
                                width,
                                dest,
                                base,
                                offset,
                            } => v.push(LI::Load {
                                width,
                                dest: r(dest),
                                offset,
                                base: r(base),
                            }),
                            AI::Store {
                                width,
                                src,
                                base,
                                offset,
                            } => v.push(LI::Store {
                                width,
                                src: r(src),
                                offset,
                                base: r(base),
                            }),
                            AI::Op {
                                op,
                                funct,
                                dest,
                                src1,
                                src2,
                            } => v.push(LI::Op {
                                op,
                                funct,
                                dest: r(dest),
                                src1: r(src1),
                                src2: r(src2),
                            }),
                            AI::OpImm { op, dest, src, imm } => v.push(LI::OpImm {
                                op,
                                dest: r(dest),
                                src: r(src),
                                imm,
                            }),
                            AI::OpImmF7 {
                                op,
                                funct,
                                dest,
                                src,
                                imm,
                            } => v.push(LI::OpImmF7 {
                                op,
                                funct,
                                dest: r(dest),
                                src: r(src),
                                imm,
                            }),
                            AI::Ret => v.push(LI::Ret),
                            AI::Call(..) => todo!(),
                            AI::Jal { .. } => todo!(),
                            AI::J(..) => todo!(),
                            AI::Branch { .. } => todo!(),
                        }
                    }
                    for (reg, var) in outputs {
                        v.push(LI::sd(*reg, stack[var], sp));
                    }
                }
                IRI::Ret { src } => {
                    if let Some(src) = src {
                        let Some(rva) = stack_rva else {
                            panic!("no return value address on stack!")
                        };
                        v.push(LI::ld(t0, rva, sp));
                        mov_mem(&mut v, sp, stack[src], t0, 0, t1, align(&f.ret_size) as u32);
                    }
                    v.extend(&ret);
                }
                IRI::Jump(location) => {
                    v.push(LI::J(*location));
                }
                IRI::Branch { to, cond } => {
                    v.push(LI::ld(t0, stack[cond], sp));
                    v.push(LI::Branch {
                        to: *to,
                        typ: branch::EQ,
                        left: t0,
                        right: zero,
                    })
                }
                IRI::Mark(location) => {
                    locations.insert(v.len(), *location);
                }
            }
        }
        dbg.push_fn(irli);
        fns.push(UnlinkedFunction {
            instrs: v,
            sym: *sym,
            locations,
        });
    }
    UnlinkedProgram {
        fns,
        ro_data: data,
        start: Some(program.entry()),
        dbg,
        sym_count: program.len(),
    }
}
