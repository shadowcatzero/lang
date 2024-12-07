use super::{PAsmArg, FnLowerCtx, PIdent, Node, PInstruction};
use crate::{
    compiler::arch::riscv64::*,
    ir::{
        arch::riscv64::{RV64Instruction, RegRef},
        VarID,
    },
};

impl RV64Instruction {
    pub fn parse(inst: &PInstruction, ctx: &mut FnLowerCtx) -> Option<Self> {
        let args = &inst.args[..];
        Some(match &**inst.op.inner.as_ref()? {
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
            "ld" => {
                let [dest, offset, base] = args else {
                    ctx.err("ld requires 3 arguments".to_string());
                    return None;
                };
                let dest = RegRef::from_arg(dest, ctx)?;
                let offset = i64_from_arg(offset, ctx)?;
                let base = RegRef::from_arg(base, ctx)?;
                Self::Ld { dest, offset, base }
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
            w => {
                ctx.err_at(inst.op.span, format!("Unknown instruction '{}'", w));
                return None;
            }
        })
    }
}

pub fn arg_to_var(node: &Node<PAsmArg>, ctx: &mut FnLowerCtx) -> Option<VarID> {
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
                let Some(reg) = Reg::from_ident(node, ctx) else {
                    return None;
                };
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
        ctx.err_at(node.span, format!("Expected an i64, found reference"));
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
