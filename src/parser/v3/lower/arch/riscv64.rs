use super::{FnLowerCtx, Node, PAsmArg, PIdent, PInstruction};
use crate::{
    compiler::arch::riscv::*,
    ir::{
        arch::riscv64::{RV64Instruction, RegRef},
        VarInst,
    },
};

impl RV64Instruction {
    pub fn parse(inst: &PInstruction, ctx: &mut FnLowerCtx) -> Option<Self> {
        let args = &inst.args[..];
        let opstr = &**inst.op.inner.as_ref()?;
        let opi = |ctx: &mut FnLowerCtx<'_, '_>, op: Funct3| -> Option<Self> {
            let [dest, src, imm] = args else {
                ctx.err(format!("{opstr} requires 3 arguments"));
                return None;
            };
            let dest = RegRef::from_arg(dest, ctx)?;
            let src = RegRef::from_arg(src, ctx)?;
            let imm = i32_from_arg(imm, ctx)?;
            Some(Self::OpImm { op, dest, src, imm })
        };
        let op = |ctx: &mut FnLowerCtx<'_, '_>, op: Funct3, funct: Funct7| -> Option<Self> {
            let [dest, src1, src2] = args else {
                ctx.err(format!("{opstr} requires 3 arguments"));
                return None;
            };
            let dest = RegRef::from_arg(dest, ctx)?;
            let src1 = RegRef::from_arg(src1, ctx)?;
            let src2 = RegRef::from_arg(src2, ctx)?;
            Some(Self::Op {
                op,
                funct,
                dest,
                src1,
                src2,
            })
        };
        let opif7 = |ctx: &mut FnLowerCtx<'_, '_>, op: Funct3, funct: Funct7| -> Option<Self> {
            let [dest, src, imm] = args else {
                ctx.err(format!("{opstr} requires 3 arguments"));
                return None;
            };
            let dest = RegRef::from_arg(dest, ctx)?;
            let src = RegRef::from_arg(src, ctx)?;
            let imm = i32_from_arg(imm, ctx)?;
            Some(Self::OpImmF7 {
                op,
                funct,
                dest,
                src,
                imm,
            })
        };
        let store = |ctx: &mut FnLowerCtx<'_, '_>, width: Funct3| -> Option<Self> {
            let [src, offset, base] = args else {
                ctx.err(format!("{opstr} requires 3 arguments"));
                return None;
            };
            let src = RegRef::from_arg(src, ctx)?;
            let offset = i32_from_arg(offset, ctx)?;
            let base = RegRef::from_arg(base, ctx)?;
            Some(Self::Store {
                width,
                src,
                offset,
                base,
            })
        };
        let load = |ctx: &mut FnLowerCtx<'_, '_>, width: Funct3| -> Option<Self> {
            let [dest, offset, base] = args else {
                ctx.err("ld requires 3 arguments".to_string());
                return None;
            };
            let dest = RegRef::from_arg(dest, ctx)?;
            let offset = i32_from_arg(offset, ctx)?;
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
                let imm = i32_from_arg(imm, ctx)?;
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

            "lb" => load(ctx, width::B)?,
            "lh" => load(ctx, width::H)?,
            "lw" => load(ctx, width::W)?,
            "ld" => load(ctx, width::D)?,
            "lbu" => load(ctx, width::BU)?,
            "lhu" => load(ctx, width::HU)?,
            "lwu" => load(ctx, width::WU)?,

            "sb" => store(ctx, width::B)?,
            "sh" => store(ctx, width::H)?,
            "sw" => store(ctx, width::W)?,
            "sd" => store(ctx, width::D)?,

            "addi" => opi(ctx, op32i::ADD)?,
            "slti" => opi(ctx, op32i::SLT)?,
            "sltiu" => opi(ctx, op32i::SLTU)?,
            "xori" => opi(ctx, op32i::XOR)?,
            "ori" => opi(ctx, op32i::OR)?,
            "andi" => opi(ctx, op32i::AND)?,

            "slli" => opif7(ctx, op32i::SL, op32i::LOGICAL)?,
            "srli" => opif7(ctx, op32i::SR, op32i::LOGICAL)?,
            "srla" => opif7(ctx, op32i::SR, op32i::ARITHMETIC)?,

            "add" => op(ctx, op32i::ADD, op32i::F7ADD)?,
            "sub" => op(ctx, op32i::ADD, op32i::F7SUB)?,
            "sll" => op(ctx, op32i::SL, op32i::FUNCT7)?,
            "slt" => op(ctx, op32i::SLT, op32i::FUNCT7)?,
            "sltu" => op(ctx, op32i::SLTU, op32i::FUNCT7)?,
            "xor" => op(ctx, op32i::XOR, op32i::FUNCT7)?,
            "srl" => op(ctx, op32i::SR, op32i::LOGICAL)?,
            "sra" => op(ctx, op32i::SR, op32i::ARITHMETIC)?,
            "or" => op(ctx, op32i::OR, op32i::FUNCT7)?,
            "and" => op(ctx, op32i::AND, op32i::FUNCT7)?,

            "mul" => op(ctx, op32m::MUL, op32m::FUNCT7)?,
            "mulh" => op(ctx, op32m::MULH, op32m::FUNCT7)?,
            "mulhsu" => op(ctx, op32m::MULHSU, op32m::FUNCT7)?,
            "mulhu" => op(ctx, op32m::MULHU, op32m::FUNCT7)?,
            "div" => op(ctx, op32m::DIV, op32m::FUNCT7)?,
            "divu" => op(ctx, op32m::DIVU, op32m::FUNCT7)?,
            "rem" => op(ctx, op32m::REM, op32m::FUNCT7)?,
            "remu" => op(ctx, op32m::REMU, op32m::FUNCT7)?,

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

fn i32_from_arg(node: &Node<PAsmArg>, ctx: &mut FnLowerCtx) -> Option<i32> {
    let PAsmArg::Value(node) = node.inner.as_ref()? else {
        ctx.err_at(node.span, "Expected an i32, found reference".to_string());
        return None;
    };
    let word = node.inner.as_ref()?;
    match word.parse::<i32>() {
        Ok(x) => Some(x),
        Err(_) => {
            ctx.err_at(node.span, format!("Expected an i64, found {}", word));
            None
        }
    }
}
