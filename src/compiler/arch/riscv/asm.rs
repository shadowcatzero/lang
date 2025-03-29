use crate::{
    compiler::program::{Addr, Instr, SymTable},
    ir::Symbol,
    util::LabeledFmt,
};

use super::*;

#[derive(Clone, Copy)]
pub enum LinkerInstruction<R = Reg, S = Symbol> {
    Op {
        op: Funct3,
        funct: Funct7,
        dest: R,
        src1: R,
        src2: R,
    },
    OpImm {
        op: Funct3,
        dest: R,
        src: R,
        imm: i32,
    },
    OpImmF7 {
        op: Funct3,
        funct: Funct7,
        dest: R,
        src: R,
        imm: i32,
    },
    Store {
        width: Funct3,
        src: R,
        offset: i32,
        base: R,
    },
    Load {
        width: Funct3,
        dest: R,
        offset: i32,
        base: R,
    },
    Mv {
        dest: R,
        src: R,
    },
    La {
        dest: R,
        src: S,
    },
    Jal {
        dest: R,
        offset: i32,
    },
    Call(S),
    J(S),
    Branch {
        to: S,
        typ: Funct3,
        left: R,
        right: R,
    },
    Ret,
    ECall,
    EBreak,
    Li {
        dest: R,
        imm: i32,
    },
}

pub fn addi(dest: Reg, src: Reg, imm: BitsI32<11, 0>) -> RawInstruction {
    opi(op32i::ADD, dest, src, imm)
}

impl Instr for LinkerInstruction {
    fn push_to(
        &self,
        data: &mut Vec<u8>,
        sym_map: &mut SymTable,
        pos: Addr,
        missing: bool,
    ) -> Option<Symbol> {
        let last = match self {
            Self::Op {
                op,
                funct,
                dest,
                src1,
                src2,
            } => opr(*op, *funct, *dest, *src1, *src2),
            Self::OpImm { op, dest, src, imm } => opi(*op, *dest, *src, BitsI32::new(*imm)),
            Self::OpImmF7 {
                op,
                funct,
                dest,
                src,
                imm,
            } => opif7(*op, *funct, *dest, *src, BitsI32::new(*imm)),
            Self::Store {
                width,
                src,
                offset,
                base,
            } => store(*width, *src, BitsI32::new(*offset), *base),
            Self::Load {
                width,
                dest,
                offset,
                base,
            } => load(*width, *dest, BitsI32::new(*offset), *base),
            Self::Mv { dest, src } => addi(*dest, *src, BitsI32::new(0)),
            Self::La { dest, src } => {
                if let Some(addr) = sym_map.get(*src) {
                    let offset = addr.val() as i32 - pos.val() as i32;
                    data.extend(auipc(*dest, BitsI32::new(0)).to_le_bytes());
                    addi(*dest, *dest, BitsI32::new(offset))
                } else {
                    data.extend_from_slice(&[0; 2 * 4]);
                    return Some(*src);
                }
            }
            Self::Jal { dest, offset } => jal(*dest, BitsI32::new(*offset)),
            Self::J(sym) => {
                if let Some(addr) = sym_map.get(*sym) {
                    let offset = addr.val() as i32 - pos.val() as i32;
                    j(BitsI32::new(offset))
                } else {
                    data.extend_from_slice(&[0; 4]);
                    return Some(*sym);
                }
            }
            Self::Call(sym) => {
                if let Some(addr) = sym_map.get(*sym) {
                    let offset = addr.val() as i32 - pos.val() as i32;
                    jal(ra, BitsI32::new(offset))
                } else {
                    data.extend_from_slice(&[0; 4]);
                    return Some(*sym);
                }
            }
            Self::Ret => ret(),
            Self::ECall => ecall(),
            Self::EBreak => ebreak(),
            Self::Li { dest, imm } => addi(*dest, zero, BitsI32::new(*imm)),
            Self::Branch {
                to,
                typ,
                left,
                right,
            } => {
                if let Some(addr) = sym_map.get(*to) {
                    let offset = addr.val() as i32 - pos.val() as i32;
                    branch(*typ, *left, *right, BitsI32::new(offset))
                } else {
                    data.extend_from_slice(&[0; 4]);
                    return Some(*to);
                }
            }
        };
        data.extend(last.to_le_bytes());
        None
    }
}

impl LinkerInstruction {
    pub fn addi(dest: Reg, src: Reg, imm: i32) -> Self {
        Self::OpImm {
            op: op32i::ADD,
            dest,
            src,
            imm,
        }
    }
    pub fn sd(src: Reg, offset: i32, base: Reg) -> Self {
        Self::Store {
            width: width::D,
            src,
            offset,
            base,
        }
    }
    pub fn ld(dest: Reg, offset: i32, base: Reg) -> Self {
        Self::Load {
            width: width::D,
            dest,
            offset,
            base,
        }
    }
}

// this is not even remotely worth it but technically it doesn't use the heap I think xdddddddddd
impl<R: std::fmt::Debug, S: std::fmt::Debug> std::fmt::Debug for LinkerInstruction<R, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_label(f, &|f, s| write!(f, "{s:?}"))
    }
}

pub struct DebugInstr<'a, R, S, L: Fn(&mut std::fmt::Formatter<'_>, &S) -> std::fmt::Result> {
    instr: &'a LinkerInstruction<R, S>,
    label: &'a L,
}

impl<R: std::fmt::Debug, S: std::fmt::Debug> LabeledFmt<S> for LinkerInstruction<R, S> {
    fn fmt_label(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        label: &dyn crate::util::Labeler<S>,
    ) -> std::fmt::Result {
        match self {
            Self::ECall => write!(f, "ecall"),
            Self::EBreak => write!(f, "ebreak"),
            Self::Li { dest, imm } => write!(f, "li {dest:?}, {imm:?}"),
            Self::Mv { dest, src } => write!(f, "mv {dest:?}, {src:?}"),
            Self::La { dest, src } => {
                write!(f, "la {dest:?}, @")?;
                label(f, src)
            }
            Self::Load {
                width,
                dest,
                offset,
                base,
            } => write!(f, "l{} {dest:?}, {offset}({base:?})", width::str(*width)),
            Self::Store {
                width,
                src,
                offset,
                base,
            } => write!(f, "s{} {src:?}, {offset}({base:?})", width::str(*width)),
            Self::Op {
                op,
                funct,
                dest,
                src1,
                src2,
            } => write!(f, "{} {dest:?}, {src1:?}, {src2:?}", opstr(*op, *funct)),
            Self::OpImm { op, dest, src, imm } => {
                write!(f, "{}i {dest:?}, {src:?}, {imm}", opstr(*op, op32i::FUNCT7))
            }
            Self::OpImmF7 {
                op,
                funct,
                dest,
                src,
                imm,
            } => write!(f, "{}i {dest:?}, {src:?}, {imm}", opstr(*op, *funct)),
            Self::Jal { dest, offset } => write!(f, "jal {dest:?}, {offset:?}"),
            Self::Call(s) => {
                write!(f, "call ")?;
                label(f, s)
            }
            Self::J(s) => {
                write!(f, "j ")?;
                label(f, s)
            }
            Self::Branch {
                to,
                typ,
                left,
                right,
            } => write!(f, "b{} {left:?} {right:?} {to:?}", branch::str(*typ)),
            Self::Ret => write!(f, "ret"),
        }
    }
}
