// TODO: move this into ir, not parser
use super::{Type, UInstrInst, UInstruction, UProgram};
use crate::common::{CompilerMsg, CompilerOutput, FileSpan};

impl UProgram {
    pub fn validate(&self, output: &mut CompilerOutput) {
        for (id, f) in self.iter_fns() {
            self.validate_fn(
                &f.instructions,
                self.origins.get(id),
                &f.ret,
                output,
                true,
                false,
            );
        }
        for (id, var) in self.iter_vars() {
            if var.ty == Type::Error {
                output.err(CompilerMsg {
                    msg: format!("Var {:?} is error type!", id),
                    spans: vec![self.origins.get(id)],
                });
            }
            if var.ty == Type::Infer {
                output.err(CompilerMsg {
                    msg: format!("Var {:?} cannot be inferred!", id),
                    spans: vec![self.origins.get(id)],
                });
            }
            if var.ty == Type::Placeholder {
                output.err(CompilerMsg {
                    msg: format!("Var {:?} still placeholder!", id),
                    spans: vec![self.origins.get(id)],
                });
            }
            if let Some(parent) = &var.parent {
                let pty = &self.get(parent.var).unwrap().ty;
                if let Some(ft) = self.field_type(pty, &parent.field) {
                    output.check_assign(self, &var.ty, ft, self.origins.get(id));
                } else {
                    output.err(CompilerMsg {
                        msg: "invalid parent!".to_string(),
                        spans: vec![self.origins.get(id)],
                    });
                }
            }
        }
    }

    pub fn validate_fn(
        &self,
        instructions: &[UInstrInst],
        origin: FileSpan,
        ret: &Type,
        output: &mut CompilerOutput,
        needs_ret: bool,
        breakable: bool,
    ) {
        let mut no_ret = true;
        for i in instructions {
            match &i.i {
                UInstruction::Mv { dest, src } => {
                    let dest = self.expect(dest.id);
                    let src = self.expect(src.id);
                    output.check_assign(self, &src.ty, &dest.ty, i.span);
                }
                UInstruction::Ref { dest, src } => {
                    let dest = self.expect(dest.id);
                    let src = self.expect(src.id);
                    output.check_assign(self, &src.ty.clone().rf(), &dest.ty, i.span);
                }
                UInstruction::LoadData { dest, src } => {
                    let dest = self.expect(dest.id);
                    let src = self.expect(*src);
                    output.check_assign(self, &src.ty, &dest.ty, i.span);
                }
                UInstruction::LoadSlice { dest, src } => {
                    let dest = self.expect(dest.id);
                    let src = self.expect(*src);
                    let Type::Array(srcty, ..) = &src.ty else {
                        todo!()
                    };
                    output.check_assign(self, &Type::Slice(srcty.clone()), &dest.ty, i.span);
                }
                UInstruction::LoadFn { dest, src } => todo!(),
                UInstruction::Call { dest, f, args } => {
                    let destty = &self.expect(dest.id).ty;
                    let f = self.expect(f.id);
                    let Type::Fn { args: argtys, ret } = &f.ty else {
                        output.err(CompilerMsg {
                            msg: format!("Type {} is not callable", self.type_name(&f.ty)),
                            spans: vec![dest.span],
                        });
                        continue;
                    };
                    output.check_assign(self, ret, destty, dest.span);
                    if args.len() != argtys.len() {
                        output.err(CompilerMsg {
                            msg: "Wrong number of arguments to function".to_string(),
                            spans: vec![dest.span],
                        });
                    }
                    for (dst_ty, src) in argtys.iter().zip(args) {
                        let src_var = self.expect(src.id);
                        output.check_assign(self, &src_var.ty, dst_ty, src.span);
                    }
                }
                UInstruction::AsmBlock { instructions, args } => {
                    for arg in args {
                        // TODO: validate size with enabled targets
                        // if let Some(size) = self.size_of_var(arg.var.id)
                        //     && size != 64
                        // {
                        //     output.err(CompilerMsg {
                        //         msg: format!("asm block args must be size 64, is size {}", size),
                        //         spans: vec![arg.var.span],
                        //     });
                        // }
                    }
                }
                UInstruction::Ret { src } => {
                    let srcty = &self.expect(src.id).ty;
                    output.check_assign(self, srcty, ret, src.span);
                    no_ret = false;
                }
                UInstruction::Construct { dest, fields } => {
                    let dest_def = self.expect(dest.id);
                    let (tyid, args) = match &dest_def.ty {
                        Type::Struct { id, args } => (*id, args),
                        _ => {
                            output.err(CompilerMsg {
                                msg: format!(
                                    "Type {} cannot be constructed",
                                    self.type_name(&dest_def.ty)
                                ),
                                spans: vec![dest.span],
                            });
                            continue;
                        }
                    };
                    let def = self.expect(tyid);
                    for (name, field) in &def.fields {
                        if let Some(var) = fields.get(name) {
                            let mut sty = &field.ty;
                            if let Type::Generic { id } = sty {
                                for (g, a) in def.generics.iter().zip(args) {
                                    if *g == *id {
                                        sty = a;
                                    }
                                }
                            }
                            let ety = &self.expect(var.id).ty;
                            output.check_assign(self, ety, sty, var.span);
                        } else {
                            output.err(CompilerMsg {
                                msg: format!("field '{}' missing from struct", name),
                                spans: vec![dest.span],
                            });
                        }
                    }
                }
                UInstruction::If { cond, body } => {
                    let cond = self.expect(cond.id);
                    output.check_assign(self, &cond.ty, &Type::Bits(64), i.span);
                    self.validate_fn(body, origin, ret, output, false, breakable);
                }
                UInstruction::Loop { body } => {
                    self.validate_fn(body, origin, ret, output, false, true);
                }
                UInstruction::Break => {
                    if !breakable {
                        output.err(CompilerMsg {
                            msg: "Can't break here (outside of loop)".to_string(),
                            spans: vec![i.span],
                        });
                    }
                    // TODO
                }
                UInstruction::Continue => {
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
