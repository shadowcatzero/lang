// TODO: move this into ir, not parser
use super::{IRUProgram, Type};
use crate::common::{CompilerMsg, CompilerOutput};

impl IRUProgram {
    pub fn validate(&self) -> CompilerOutput {
        let mut output = CompilerOutput::new();
        for (f, fd) in self.fns.iter().flatten().zip(&self.fn_defs) {
            for i in &f.instructions {
                match &i.i {
                    super::IRUInstruction::Mv { dest, src } => {
                        let dest = self.get_var(dest.id);
                        let src = self.get_var(src.id);
                        output.check_assign(self, &src.ty, &dest.ty, i.span);
                    }
                    super::IRUInstruction::Ref { dest, src } => todo!(),
                    super::IRUInstruction::LoadData { dest, src } => {
                        let dest = self.get_var(dest.id);
                        let src = self.get_data(*src);
                        output.check_assign(self, &src.ty, &dest.ty, i.span);
                    }
                    super::IRUInstruction::LoadSlice { dest, src } => {
                        let dest = self.get_var(dest.id);
                        let src = self.get_data(*src);
                        let Type::Array(srcty, ..) = &src.ty else {
                            todo!()
                        };
                        output.check_assign(self, &Type::Slice(srcty.clone()), &dest.ty, i.span);
                    }
                    super::IRUInstruction::LoadFn { dest, src } => todo!(),
                    super::IRUInstruction::Call { dest, f, args } => {
                        let destty = &self.get_var(dest.id).ty;
                        let f = self.get_var(f.id);
                        let Type::Fn { args: argtys, ret } = &f.ty else {
                            todo!()
                        };
                        output.check_assign(self, ret, destty, dest.span);
                        if args.len() != argtys.len() {
                            output.err(CompilerMsg {
                                msg: "Wrong number of arguments to function".to_string(),
                                spans: vec![dest.span],
                            });
                        }
                        for (argv, argt) in args.iter().zip(argtys) {
                            let dest = self.get_var(argv.id);
                            output.check_assign(self, argt, &dest.ty, argv.span);
                        }
                    }
                    super::IRUInstruction::AsmBlock { instructions, args } => {
                        // TODO
                    }
                    super::IRUInstruction::Ret { src } => {
                        let srcty = &self.get_var(src.id).ty;
                        output.check_assign(self, srcty, &fd.ret, src.span);
                    },
                }
            }
        }
        output
    }
}
