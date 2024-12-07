use crate::{
    compiler::arch::riscv64::Reg,
    ir::{arch::riscv64::RV64Instruction, IRUInstruction, VarID},
};

use super::{PAsmBlock, PAsmBlockArg, FnLowerCtx, FnLowerable, PInstruction};

impl FnLowerable for PInstruction {
    type Output = RV64Instruction;

    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<RV64Instruction> {
        RV64Instruction::parse(self, ctx)
    }
}

impl FnLowerable for PAsmBlock {
    type Output = ();

    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<Self::Output> {
        let block = IRUInstruction::AsmBlock {
            instructions: {
                let mut v = Vec::new();
                for i in &self.instructions {
                    if let Some(i) = i.lower(ctx) {
                        v.push(i);
                    }
                }
                v
            },
            args: {
                let mut v = Vec::new();
                for a in &self.args {
                    if let Some(a) = a.lower(ctx) {
                        v.push(a);
                    }
                }
                v
            },
        };
        ctx.push(block);
        Some(())
    }
}

impl FnLowerable for PAsmBlockArg {
    type Output = (Reg, VarID);

    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<Self::Output> {
        let var = ctx.get_var(&self.var)?;
        let reg = Reg::from_ident(&self.reg, ctx)?;
        Some((reg, var))
    }
}
