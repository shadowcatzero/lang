use crate::{
    compiler::arch::riscv::Reg,
    ir::{
        arch::riscv64::RV64Instruction, AsmBlockArg, AsmBlockArgType, Type, UInstruction, VarInst,
        VarInstID,
    },
    parser::PAsmBlockArg,
};

use super::{FnLowerCtx, FnLowerable, PAsmBlock, PInstruction, PUAsmBlockArg};

type PLAsmBlockArg = PAsmBlockArg<Reg, VarInstID>;

impl FnLowerable for PInstruction {
    type Output = RV64Instruction;

    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<RV64Instruction> {
        RV64Instruction::parse(self, ctx)
    }
}

impl FnLowerable for PAsmBlock {
    type Output = VarInstID;

    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<Self::Output> {
        let mut args = Vec::new();
        let mut output = None;
        for a in &self.args {
            if let Some(a) = a.lower(ctx) {
                match a {
                    PAsmBlockArg::In { reg, var } => args.push(AsmBlockArg {
                        reg,
                        var,
                        ty: AsmBlockArgType::In,
                    }),
                    PAsmBlockArg::Out { reg } => {
                        if output.is_some() {
                            ctx.err("cannot evaluate to more than one register".to_string());
                            continue;
                        }
                        let var = ctx.temp(Type::Bits(64));
                        args.push(AsmBlockArg {
                            var,
                            reg,
                            ty: AsmBlockArgType::Out,
                        });
                        output = Some(var)
                    }
                }
            }
        }
        let block = UInstruction::AsmBlock {
            instructions: {
                let mut v = Vec::new();
                for i in &self.instructions {
                    if let Some(i) = i.lower(ctx) {
                        v.push(i);
                    }
                }
                v
            },
            args,
        };
        ctx.push(block);
        output
    }
}

impl FnLowerable for PUAsmBlockArg {
    type Output = PLAsmBlockArg;

    fn lower(&self, ctx: &mut FnLowerCtx) -> Option<Self::Output> {
        Some(match self {
            PAsmBlockArg::In { reg, var } => PLAsmBlockArg::In {
                reg: Reg::from_ident(reg, ctx)?,
                var: var.as_ref()?.lower(ctx)?,
            },
            PAsmBlockArg::Out { reg } => PLAsmBlockArg::Out {
                reg: Reg::from_ident(reg, ctx)?,
            },
        })
    }
}
