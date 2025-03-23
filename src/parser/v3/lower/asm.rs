use crate::{
    compiler::arch::riscv::Reg,
    ir::{arch::riscv64::RV64Instruction, IRUInstruction, VarInst},
};

use super::{FnLowerCtx, FnLowerable, PAsmBlock, PAsmBlockArg, PInstruction};

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
    type Output = (Reg, VarInst);

    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<Self::Output> {
        let var = ctx.get_var(&self.var)?;
        let reg = Reg::from_ident(&self.reg, ctx)?;
        Some((reg, var))
    }
}
