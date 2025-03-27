// TODO: move this into ir, not parser
use super::{IRUInstruction, IRUProgram, Type};
use crate::common::{CompilerMsg, CompilerOutput};

impl IRUProgram {
    pub fn validate(&self) -> CompilerOutput {
        let mut output = CompilerOutput::new();
        for (f, fd) in self.fns.iter().flatten().zip(&self.fn_defs) {
            for i in &f.instructions {
                match &i.i {
                    IRUInstruction::Mv { dest, src } => {
                        let dest = self.get_var(dest.id);
                        let src = self.get_var(src.id);
                        output.check_assign(self, &src.ty, &dest.ty, i.span);
                    }
                    IRUInstruction::Ref { dest, src } => todo!(),
                    IRUInstruction::LoadData { dest, src } => {
                        let dest = self.get_var(dest.id);
                        let src = self.get_data(*src);
                        output.check_assign(self, &src.ty, &dest.ty, i.span);
                    }
                    IRUInstruction::LoadSlice { dest, src } => {
                        let dest = self.get_var(dest.id);
                        let src = self.get_data(*src);
                        let Type::Array(srcty, ..) = &src.ty else {
                            todo!()
                        };
                        output.check_assign(self, &Type::Slice(srcty.clone()), &dest.ty, i.span);
                    }
                    IRUInstruction::LoadFn { dest, src } => todo!(),
                    IRUInstruction::Call { dest, f, args } => {
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
                    IRUInstruction::AsmBlock { instructions, args } => {
                        // TODO
                    }
                    IRUInstruction::Ret { src } => {
                        let srcty = &self.get_var(src.id).ty;
                        output.check_assign(self, srcty, &fd.ret, src.span);
                    }
                    IRUInstruction::Construct { dest, fields } => {
                        let dest_def = self.get_var(dest.id);
                        let tyid = match dest_def.ty {
                            Type::Concrete(id) => id,
                            _ => {
                                output.err(CompilerMsg {
                                    msg: "uhh type is not struct".to_string(),
                                    spans: vec![dest.span],
                                });
                                continue;
                            }
                        };
                        let def = self.get_struct(tyid);
                        for (name, field) in &def.fields {
                            if let Some(var) = fields.get(name) {
                                let ety = &self.get_var(var.id).ty;
                                output.check_assign(self, &field.ty, ety, var.span);
                            } else {
                                output.err(CompilerMsg {
                                    msg: format!("field '{name}' missing from struct"),
                                    spans: vec![dest.span],
                                });
                            }
                        }
                        for name in fields.keys() {
                            if !def.fields.contains_key(name) {
                                output.err(CompilerMsg {
                                    msg: format!("field '{name}' not in struct"),
                                    spans: vec![dest.span],
                                });
                            }
                        }
                    }
                    IRUInstruction::Access { dest, src, field } => {
                        let dest_def = self.get_var(dest.id);
                        let src_def = self.get_var(src.id);
                        let tyid = match src_def.ty {
                            Type::Concrete(id) => id,
                            _ => {
                                output.err(CompilerMsg {
                                    msg: "uhh type is not struct".to_string(),
                                    spans: vec![dest.span],
                                });
                                continue;
                            }
                        };
                        let def = self.get_struct(tyid);
                        let field = def.fields.get(field).expect(
                            "already validated during parse lowering... probably shouldn't be?",
                        );
                        output.check_assign(self, &field.ty, &dest_def.ty, i.span);
                        // TODO
                    }
                }
            }
        }
        output
    }
}
