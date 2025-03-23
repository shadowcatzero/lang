use crate::{compiler::arch::riscv64::*, ir::VarInst};

#[derive(Copy, Clone)]
pub enum RV64Instruction {
    Ecall,
    Li {
        dest: RegRef,
        imm: i64,
    },
    Mv {
        dest: RegRef,
        src: RegRef,
    },
    La {
        dest: RegRef,
        src: VarInst,
    },
    Load {
        width: Width,
        dest: RegRef,
        offset: i64,
        base: RegRef,
    },
    Store {
        width: Width,
        src: RegRef,
        offset: i64,
        base: RegRef,
    },
    Op {
        op: Op,
        dest: RegRef,
        src1: RegRef,
        src2: RegRef,
    },
    OpF7 {
        op: Op,
        funct: Funct7,
        dest: RegRef,
        src1: RegRef,
        src2: RegRef,
    },
    OpImm {
        op: Op,
        dest: RegRef,
        src: RegRef,
        imm: i64,
    },
    OpImmF7 {
        op: Op,
        funct: Funct7,
        dest: RegRef,
        src: RegRef,
        imm: i64,
    },
}

#[derive(Copy, Clone)]
pub enum RegRef {
    Var(VarInst),
    Reg(Reg),
}

impl std::fmt::Debug for RegRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Var(v) => write!(f, "{{{:?}}}", v),
            Self::Reg(r) => r.fmt(f),
        }
    }
}

impl std::fmt::Debug for RV64Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ecall => write!(f, "ecall"),
            Self::Li { dest, imm } => write!(f, "li {dest:?}, {imm:?}"),
            Self::Mv { dest, src } => write!(f, "mv {dest:?}, {src:?}"),
            Self::La { dest, src } => write!(f, "la {dest:?}, {src:?}"),
            Self::Load {
                width,
                dest,
                offset,
                base,
            } => write!(f, "l{} {dest:?}, {offset}({base:?})", width.str()),
            Self::Store {
                width,
                src,
                offset,
                base,
            } => write!(f, "s{} {src:?}, {offset}({base:?})", width.str()),
            Self::Op {
                op,
                dest,
                src1,
                src2,
            } => write!(f, "{} {dest:?}, {src1:?}, {src2:?}", op.str()),
            Self::OpF7 {
                op,
                funct,
                dest,
                src1,
                src2,
            } => write!(f, "{} {dest:?}, {src1:?}, {src2:?}", op.fstr(*funct)),
            Self::OpImm { op, dest, src, imm } => {
                write!(f, "{}i {dest:?}, {src:?}, {imm}", op.str())
            }
            Self::OpImmF7 {
                op,
                funct,
                dest,
                src,
                imm,
            } => write!(f, "{}i {dest:?}, {src:?}, {imm}", op.fstr(*funct)),
        }
    }
}
