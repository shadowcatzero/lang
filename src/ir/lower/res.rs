use crate::ir::{
    arch::riscv64::{RV64Instruction, RegRef},
    AsmBlockArg, Resolved, UInstrInst, UInstruction, UProgram, VarID,
};

impl UInstrInst {
    pub fn resolve<'a>(&'a self, p: &'a UProgram) -> Option<UInstrInst<Resolved>> {
        Some(UInstrInst {
            i: self.i.resolve(p)?,
            origin: self.origin,
        })
    }
}

impl UInstruction {
    pub fn resolve<'a>(&'a self, p: &'a UProgram) -> Option<UInstruction<Resolved>> {
        use UInstruction as I;
        Some(match self {
            I::Mv { dst, src } => I::Mv {
                dst: dst.var(p)?,
                src: src.var(p)?,
            },
            I::Ref { dst, src } => I::Ref {
                dst: dst.var(p)?,
                src: src.var(p)?,
            },
            I::Deref { dst, src } => I::Deref {
                dst: dst.var(p)?,
                src: src.var(p)?,
            },
            I::LoadData { dst, src } => I::LoadData {
                dst: dst.var(p)?,
                src: *src,
            },
            I::LoadSlice { dst, src } => I::LoadSlice {
                dst: dst.var(p)?,
                src: *src,
            },
            I::Call { dst, f, args } => I::Call {
                dst: dst.var(p)?,
                f: f.fun(p)?.clone(),
                args: args.iter().map(|i| i.var(p)).try_collect()?,
            },
            I::AsmBlock { instructions, args } => I::AsmBlock {
                instructions: instructions
                    .iter()
                    .map(|i| i.resolve(p))
                    .collect::<Option<_>>()?,
                args: args.iter().map(|a| a.resolve(p)).try_collect()?,
            },
            I::Ret { src } => I::Ret { src: src.var(p)? },
            I::Construct { dst, struc, fields } => I::Construct {
                dst: dst.var(p)?,
                struc: struc.struc(p)?.clone(),
                fields: fields
                    .iter()
                    .map(|(name, ident)| ident.var(p).map(|i| (name.clone(), i)))
                    .collect::<Option<_>>()?,
            },
            I::If { cond, body } => I::If {
                cond: cond.var(p)?,
                body: body.iter().map(|i| i.resolve(p)).try_collect()?,
            },
            I::Loop { body } => I::Loop {
                body: body.iter().map(|i| i.resolve(p)).try_collect()?,
            },
            I::Break => I::Break,
            I::Continue => I::Continue,
        })
    }
}

impl AsmBlockArg {
    pub fn resolve(&self, p: &UProgram) -> Option<AsmBlockArg<VarID>> {
        Some(AsmBlockArg {
            var: self.var.var(p)?,
            reg: self.reg,
            ty: self.ty,
        })
    }
}

impl RV64Instruction {
    pub fn resolve(&self, p: &UProgram) -> Option<RV64Instruction<VarID>> {
        self.try_map(|i| {
            Some(match i {
                RegRef::Var(v) => RegRef::Var(v.var(p)?),
                RegRef::Reg(r) => RegRef::Reg(*r),
            })
        })
    }
}
