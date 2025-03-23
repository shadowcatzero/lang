use crate::{
    compiler::program::{Addr, Instr, SymTable},
    ir::Symbol,
};

use super::*;

#[derive(Debug, Clone)]
pub enum LinkerInstruction {
    Op {
        op: Op,
        dest: Reg,
        src1: Reg,
        src2: Reg,
    },
    OpF7 {
        op: Op,
        funct: Funct7,
        dest: Reg,
        src1: Reg,
        src2: Reg,
    },
    OpImm {
        op: Op,
        dest: Reg,
        src: Reg,
        imm: i32,
    },
    OpImmF7 {
        op: Op,
        funct: Funct7,
        dest: Reg,
        src: Reg,
        imm: i32,
    },
    Store {
        width: Width,
        src: Reg,
        offset: i32,
        base: Reg,
    },
    Load {
        width: Width,
        dest: Reg,
        offset: i32,
        base: Reg,
    },
    Mv {
        dest: Reg,
        src: Reg,
    },
    La {
        dest: Reg,
        src: Symbol,
    },
    Jal {
        dest: Reg,
        offset: i32,
    },
    Call(Symbol),
    J(Symbol),
    Ret,
    Ecall,
    Li {
        dest: Reg,
        imm: i64,
    },
}

pub const fn addi(dest: Reg, src: Reg, imm: BitsI32<11, 0>) -> RawInstruction {
    opi(Op::Add, dest, src, imm)
}

impl Instr for LinkerInstruction {
    fn push(
        &self,
        data: &mut Vec<u8>,
        sym_map: &SymTable,
        pos: Addr,
        missing: bool,
    ) -> Option<Symbol> {
        let last = match self {
            Self::Op {
                op,
                dest,
                src1,
                src2,
            } => opr(*op, *dest, *src1, *src2),
            Self::OpF7 {
                op,
                funct,
                dest,
                src1,
                src2,
            } => oprf7(*op, *funct, *dest, *src1, *src2),
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
            Self::Ecall => ecall(),
            Self::Li { dest, imm } => addi(*dest, zero, BitsI32::new(*imm as i32)),
        };
        data.extend(last.to_le_bytes());
        None
    }
}

impl LinkerInstruction {
    pub fn addi(dest: Reg, src: Reg, imm: i32) -> Self {
        Self::OpImm {
            op: Op::Add,
            dest,
            src,
            imm,
        }
    }
    pub fn sd(src: Reg, offset: i32, base: Reg) -> Self {
        Self::Store {
            width: Width::D,
            src,
            offset,
            base,
        }
    }
    pub fn ld(dest: Reg, offset: i32, base: Reg) -> Self {
        Self::Load {
            width: Width::D,
            dest,
            offset,
            base,
        }
    }
}
