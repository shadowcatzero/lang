// TODO: move this into ir, not parser
use super::{IRUInstrInst, IRUInstruction, IRUProgram, Type};
use crate::common::{CompilerMsg, CompilerOutput, FileSpan};

impl IRUProgram {
    pub fn validate(&self) -> CompilerOutput {
        let mut output = CompilerOutput::new();
        for (f, fd) in self.fns.iter().flatten().zip(&self.fn_defs) {
            self.validate_fn(
                &f.instructions,
                fd.origin,
                &fd.ret,
                &mut output,
                true,
                false,
            );
        }
        output
    }

    pub fn validate_fn(
        &self,
        instructions: &[IRUInstrInst],
        origin: FileSpan,
        ret: &Type,
        output: &mut CompilerOutput,
        needs_ret: bool,
        breakable: bool,
    ) {
        let mut no_ret = true;
        for i in instructions {
            match &i.i {
                IRUInstruction::Mv { dest, src } => {
                    let dest = self.get_var(dest.id);
                    let src = self.get_var(src.id);
                    output.check_assign(self, &src.ty, &dest.ty, i.span);
                }
                IRUInstruction::Ref { dest, src } => {
                    // TODO
                }
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
                    output.check_assign(self, srcty, ret, src.span);
                    no_ret = false;
                }
                IRUInstruction::Construct { dest, fields } => {
                    let dest_def = self.get_var(dest.id);
                    let tyid = match &dest_def.ty {
                        Type::Struct { id, args } => *id,
                        _ => {
                            output.err(CompilerMsg {
                                msg: "uhh type is not struct".to_string(),
                                spans: vec![dest.span],
                            });
                            continue;
                        }
                    };
                    let def = self.get_struct(tyid);
                    for (id, field) in def.iter_fields() {
                        if let Some(var) = fields.get(&id) {
                            let ety = &self.get_var(var.id).ty;
                            output.check_assign(self, &field.ty, ety, var.span);
                        } else {
                            output.err(CompilerMsg {
                                msg: format!("field '{}' missing from struct", field.name),
                                spans: vec![dest.span],
                            });
                        }
                    }
                }
                IRUInstruction::If { cond, body } => {
                    let cond = self.get_var(cond.id);
                    output.check_assign(self, &cond.ty, &Type::Bits(64), i.span);
                    self.validate_fn(body, origin, ret, output, false, breakable);
                }
                IRUInstruction::Loop { body } => {
                    self.validate_fn(body, origin, ret, output, false, true);
                }
                IRUInstruction::Break => {
                    if !breakable {
                        output.err(CompilerMsg {
                            msg: "Can't break here (outside of loop)".to_string(),
                            spans: vec![i.span],
                        });
                    }
                    // TODO
                }
                IRUInstruction::Continue => {
                    if !breakable {
                        output.err(CompilerMsg {
                            msg: "Can't continue here (outside of loop)".to_string(),
                            spans: vec![i.span],
                        });
                    }
                    // TODO
                }
            }
        }
        if needs_ret && no_ret && *ret != Type::Unit {
            output.err(CompilerMsg {
                msg: format!(
                    "Function implicitly returns () at the end, must return {}",
                    self.type_name(ret)
                ),
                spans: vec![origin],
            });
        }
    }
}
