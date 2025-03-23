use super::{FnLowerCtx, Node, PAsmArg, PIdent, PInstruction};
use crate::{
    compiler::arch::riscv64::*,
    ir::{
        arch::riscv64::{RV64Instruction, RegRef},
        VarInst,
    },
};

impl RV64Instruction {
    pub fn parse(inst: &PInstruction, ctx: &mut FnLowerCtx) -> Option<Self> {
        let args = &inst.args[..];
        let opstr = &**inst.op.inner.as_ref()?;
        let op = |ctx: &mut FnLowerCtx<'_, '_>, op: Op| -> Option<Self> {
            let [dest, src1, src2] = args else {
                ctx.err(format!("{opstr} requires 3 arguments"));
                return None;
            };
            let dest = RegRef::from_arg(dest, ctx)?;
            let src1 = RegRef::from_arg(src1, ctx)?;
            let src2 = RegRef::from_arg(src2, ctx)?;
            Some(Self::Op {
                op,
                dest,
                src1,
                src2,
            })
        };
        let opi = |ctx: &mut FnLowerCtx<'_, '_>, op: Op| -> Option<Self> {
            let [dest, src, imm] = args else {
                ctx.err(format!("{opstr} requires 3 arguments"));
                return None;
            };
            let dest = RegRef::from_arg(dest, ctx)?;
            let src = RegRef::from_arg(src, ctx)?;
            let imm = i64_from_arg(imm, ctx)?;
            Some(Self::OpImm { op, dest, src, imm })
        };
        let opf7 = |ctx: &mut FnLowerCtx<'_, '_>, op: Op, funct: Funct7| -> Option<Self> {
            let [dest, src1, src2] = args else {
                ctx.err(format!("{opstr} requires 3 arguments"));
                return None;
            };
            let dest = RegRef::from_arg(dest, ctx)?;
            let src1 = RegRef::from_arg(src1, ctx)?;
            let src2 = RegRef::from_arg(src2, ctx)?;
            Some(Self::OpF7 {
                op,
                funct,
                dest,
                src1,
                src2,
            })
        };
        let opif7 = |ctx: &mut FnLowerCtx<'_, '_>, op: Op, funct: Funct7| -> Option<Self> {
            let [dest, src, imm] = args else {
                ctx.err(format!("{opstr} requires 3 arguments"));
                return None;
            };
            let dest = RegRef::from_arg(dest, ctx)?;
            let src = RegRef::from_arg(src, ctx)?;
            let imm = i64_from_arg(imm, ctx)?;
            Some(Self::OpImmF7 {
                op,
                funct,
                dest,
                src,
                imm,
            })
        };
        let store = |ctx: &mut FnLowerCtx<'_, '_>, width: Width| -> Option<Self> {
            let [src, offset, base] = args else {
                ctx.err(format!("{opstr} requires 3 arguments"));
                return None;
            };
            let src = RegRef::from_arg(src, ctx)?;
            let offset = i64_from_arg(offset, ctx)?;
            let base = RegRef::from_arg(base, ctx)?;
            Some(Self::Store {
                width,
                src,
                offset,
                base,
            })
        };
        let load = |ctx: &mut FnLowerCtx<'_, '_>, width: Width| -> Option<Self> {
            let [dest, offset, base] = args else {
                ctx.err("ld requires 3 arguments".to_string());
                return None;
            };
            let dest = RegRef::from_arg(dest, ctx)?;
            let offset = i64_from_arg(offset, ctx)?;
            let base = RegRef::from_arg(base, ctx)?;
            Some(Self::Load {
                width,
                dest,
                offset,
                base,
            })
        };
        Some(match opstr {
            "ecall" => Self::Ecall,
            "li" => {
                let [dest, imm] = args else {
                    ctx.err("li requires 2 arguments".to_string());
                    return None;
                };
                let dest = RegRef::from_arg(dest, ctx)?;
                let imm = i64_from_arg(imm, ctx)?;
                Self::Li { dest, imm }
            }
            "la" => {
                let [dest, src] = args else {
                    ctx.err("la requires 2 arguments".to_string());
                    return None;
                };
                let dest = RegRef::from_arg(dest, ctx)?;
                let src = arg_to_var(src, ctx)?;
                Self::La { dest, src }
            }
            "mv" => {
                let [dest, src] = args else {
                    ctx.err("la requires 2 arguments".to_string());
                    return None;
                };
                let dest = RegRef::from_arg(dest, ctx)?;
                let src = RegRef::from_arg(src, ctx)?;
                Self::Mv { dest, src }
            }

            "lb" => load(ctx, Width::B)?,
            "lh" => load(ctx, Width::H)?,
            "lw" => load(ctx, Width::W)?,
            "ld" => load(ctx, Width::D)?,
            "lbu" => load(ctx, Width::BU)?,
            "lhu" => load(ctx, Width::HU)?,
            "lwu" => load(ctx, Width::WU)?,

            "sb" => store(ctx, Width::B)?,
            "sh" => store(ctx, Width::H)?,
            "sw" => store(ctx, Width::W)?,
            "sd" => store(ctx, Width::D)?,

            "addi" => opi(ctx, Op::Add)?,
            "slti" => opi(ctx, Op::Slt)?,
            "sltiu" => opi(ctx, Op::Sltu)?,
            "xori" => opi(ctx, Op::Xor)?,
            "ori" => opi(ctx, Op::Or)?,
            "andi" => opi(ctx, Op::And)?,

            "slli" => opif7(ctx, Op::Sl, funct7::LOGICAL)?,
            "srli" => opif7(ctx, Op::Sr, funct7::LOGICAL)?,
            "srla" => opif7(ctx, Op::Sr, funct7::ARITHMETIC)?,

            "add" => opf7(ctx, Op::Add, funct7::ADD)?,
            "sub" => opf7(ctx, Op::Add, funct7::SUB)?,
            "sll" => op(ctx, Op::Sl)?,
            "slt" => op(ctx, Op::Slt)?,
            "sltu" => op(ctx, Op::Sltu)?,
            "xor" => op(ctx, Op::Xor)?,
            "srl" => opf7(ctx, Op::Sr, funct7::LOGICAL)?,
            "sra" => opf7(ctx, Op::Sr, funct7::ARITHMETIC)?,
            "or" => op(ctx, Op::Or)?,
            "and" => op(ctx, Op::And)?,

            w => {
                ctx.err_at(inst.op.span, format!("Unknown instruction '{}'", w));
                return None;
            }
        })
    }
}

pub fn arg_to_var(node: &Node<PAsmArg>, ctx: &mut FnLowerCtx) -> Option<VarInst> {
    let PAsmArg::Ref(node) = node.inner.as_ref()? else {
        ctx.err_at(
            node.span,
            "Expected variable / function reference".to_string(),
        );
        return None;
    };
    ctx.get_var(node)
}

impl RegRef {
    pub fn from_arg(node: &Node<PAsmArg>, ctx: &mut FnLowerCtx) -> Option<Self> {
        Some(match node.inner.as_ref()? {
            PAsmArg::Value(node) => {
                let reg = Reg::from_ident(node, ctx)?;
                Self::Reg(reg)
            }
            PAsmArg::Ref(node) => Self::Var(ctx.get_var(node)?),
        })
    }
}

impl Reg {
    pub fn from_ident(node: &Node<PIdent>, ctx: &mut FnLowerCtx) -> Option<Self> {
        let s = &**node.inner.as_ref()?;
        let res = Reg::from_str(s);
        if res.is_none() {
            ctx.err_at(node.span, format!("Unknown reg name '{}'", s));
        }
        res
    }
}

fn i64_from_arg(node: &Node<PAsmArg>, ctx: &mut FnLowerCtx) -> Option<i64> {
    let PAsmArg::Value(node) = node.inner.as_ref()? else {
        ctx.err_at(node.span, "Expected an i64, found reference".to_string());
        return None;
    };
    let word = node.inner.as_ref()?;
    match word.parse::<i64>() {
        Ok(x) => Some(x),
        Err(_) => {
            ctx.err_at(node.span, format!("Expected an i64, found {}", word));
            None
        }
    }
}
