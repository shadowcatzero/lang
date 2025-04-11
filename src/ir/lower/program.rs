use std::collections::HashMap;

use crate::ir::{AsmBlockArgType, UInstrInst, Size, SymbolSpace, UFunc, VarOffset};

use super::{
    IRLFunction, LInstruction, Len, Symbol, SymbolSpaceBuilder, Type, UInstruction, UProgram,
    VarID,
};

pub struct LProgram {
    sym_space: SymbolSpace,
    entry: Symbol,
}

// NOTE: there are THREE places here where I specify size (8)

impl LProgram {
    pub fn create(p: &UProgram) -> Result<Self, String> {
        let start = p
            .names
            .lookup::<UFunc>("start")
            .ok_or("no start method found")?;
        let mut ssbuilder = SymbolSpaceBuilder::with_entries(&[start]);
        let entry = ssbuilder.func(&start);
        while let Some((sym, i)) = ssbuilder.pop_fn() {
            let f = p.fns[i.0].as_ref().unwrap();
            let mut fbuilder = LFunctionBuilder::new(p, &mut ssbuilder);
            for i in &f.instructions {
                fbuilder.insert_instr(i);
            }
            if fbuilder.instrs.last().is_none_or(|i| !i.is_ret()) {
                fbuilder.instrs.push(LInstruction::Ret { src: None });
            }
            let res = fbuilder.finish(f);
            ssbuilder.write_fn(sym, res, Some(p.names.get(i).to_string()));
        }
        let sym_space = ssbuilder.finish().expect("we failed the mission");
        Ok(Self { sym_space, entry })
    }

    pub fn entry(&self) -> Symbol {
        self.entry
    }
}

pub struct LFunctionBuilder<'a> {
    program: &'a UProgram,
    builder: &'a mut SymbolSpaceBuilder,
    instrs: Vec<LInstruction>,
    stack: HashMap<VarID, Size>,
    subvar_map: HashMap<VarID, VarOffset>,
    makes_call: bool,
    loopp: Option<LoopCtx>,
}

#[derive(Clone, Copy)]
pub struct LoopCtx {
    top: Symbol,
    bot: Symbol,
}

impl<'a> LFunctionBuilder<'a> {
    pub fn new(program: &'a UProgram, builder: &'a mut SymbolSpaceBuilder) -> Self {
        Self {
            instrs: Vec::new(),
            stack: HashMap::new(),
            subvar_map: HashMap::new(),
            makes_call: false,
            program,
            builder,
            loopp: None,
        }
    }
    pub fn alloc_stack(&mut self, i: VarID) -> Option<()> {
        if self.program.size_of_var(i).expect("unsized type") == 0 {
            return None;
        };
        self.map_subvar(i);
        let var = self.program.var_offset(i).expect("var offset");
        *self
            .stack
            .entry(var.id)
            .or_insert(self.program.size_of_var(var.id).expect("unsized type"));
        Some(())
    }
    pub fn map_subvar(&mut self, i: VarID) {
        let off = self.program.var_offset(i).expect("var offset");
        if off.id != i {
            self.subvar_map.insert(i, off);
        }
    }
    pub fn insert_instr(&mut self, i: &UInstrInst) -> Option<Option<String>> {
        match &i.i {
            UInstruction::Mv { dest, src } => {
                self.alloc_stack(dest.id)?;
                self.map_subvar(src.id);
                self.instrs.push(LInstruction::Mv {
                    dest: dest.id,
                    dest_offset: 0,
                    src: src.id,
                    src_offset: 0,
                });
            }
            UInstruction::Ref { dest, src } => {
                self.alloc_stack(dest.id)?;
                self.map_subvar(src.id);
                self.instrs.push(LInstruction::Ref {
                    dest: dest.id,
                    src: src.id,
                });
            }
            UInstruction::LoadData { dest, src } => {
                self.alloc_stack(dest.id)?;
                let data = self.program.expect(*src);
                let sym = self.builder.ro_data(
                    src,
                    &data.content,
                    Some(self.program.names.get(dest.id).to_string()),
                );
                self.instrs.push(LInstruction::LoadData {
                    dest: dest.id,
                    offset: 0,
                    len: data.content.len() as Len,
                    src: sym,
                });
            }
            UInstruction::LoadSlice { dest, src } => {
                self.alloc_stack(dest.id)?;
                let data = self.program.expect(*src);
                let Type::Array(_, len) = &data.ty else {
                    return Some(Some(format!(
                        "tried to load {} as slice",
                        self.program.type_name(&data.ty)
                    )));
                };
                let sym = self.builder.ro_data(
                    src,
                    &data.content,
                    Some(self.program.names.get(dest.id).to_string()),
                );
                self.instrs.push(LInstruction::LoadAddr {
                    dest: dest.id,
                    offset: 0,
                    src: sym,
                });

                let sym = self
                    .builder
                    .anon_ro_data(&(*len as u64).to_le_bytes(), Some(format!("len: {}", len)));
                self.instrs.push(LInstruction::LoadData {
                    dest: dest.id,
                    offset: 8,
                    len: 8,
                    src: sym,
                });
            }
            UInstruction::LoadFn { dest, src } => {
                self.alloc_stack(dest.id)?;
                let sym = self.builder.func(src);
                self.instrs.push(LInstruction::LoadAddr {
                    dest: dest.id,
                    offset: 0,
                    src: sym,
                });
            }
            UInstruction::Call { dest, f, args } => {
                self.alloc_stack(dest.id);
                self.makes_call = true;
                let fid = &self.program.fn_map[&f.id];
                let sym = self.builder.func(fid);
                let ret_size = self.program.size_of_var(dest.id).expect("unsized type");
                let dest = if ret_size > 0 {
                    Some((dest.id, ret_size))
                } else {
                    None
                };
                let call = LInstruction::Call {
                    dest,
                    f: sym,
                    args: args
                        .iter()
                        .map(|a| {
                            self.map_subvar(a.id);
                            (a.id, self.program.size_of_var(a.id).expect("unsized type"))
                        })
                        .collect(),
                };
                self.instrs.push(call);
            }
            UInstruction::AsmBlock { instructions, args } => {
                let mut inputs = Vec::new();
                let mut outputs = Vec::new();
                for a in args {
                    match a.ty {
                        AsmBlockArgType::In => {
                            self.map_subvar(a.var.id);
                            inputs.push((a.reg, a.var.id))
                        }
                        AsmBlockArgType::Out => {
                            self.alloc_stack(a.var.id)?;
                            outputs.push((a.reg, a.var.id));
                        }
                    }
                }
                self.instrs.push(LInstruction::AsmBlock {
                    instructions: instructions.clone(),
                    inputs,
                    outputs,
                })
            }
            UInstruction::Ret { src } => {
                self.map_subvar(src.id);
                self.instrs.push(LInstruction::Ret {
                    src: if self.program.size_of_var(src.id).expect("unsized var") == 0 {
                        None
                    } else {
                        Some(src.id)
                    },
                })
            }
            UInstruction::Construct { dest, fields } => {
                self.alloc_stack(dest.id)?;
                let ty = &self.program.expect(dest.id).ty;
                let &Type::Struct { id, ref args } = ty else {
                    return Some(Some(format!(
                        "Failed to contruct type {}",
                        self.program.type_name(ty)
                    )));
                };
                for (&fid, var) in fields {
                    self.map_subvar(var.id);
                    self.instrs.push(LInstruction::Mv {
                        dest: dest.id,
                        src: var.id,
                        dest_offset: self.program.field_offset(id, fid).expect("field offset"),
                        src_offset: 0,
                    })
                }
            }
            UInstruction::If { cond, body } => {
                self.map_subvar(cond.id);
                let sym = self.builder.reserve();
                self.instrs.push(LInstruction::Branch {
                    to: *sym,
                    cond: cond.id,
                });
                for i in body {
                    self.insert_instr(i);
                }
                self.instrs.push(LInstruction::Mark(*sym));
            }
            UInstruction::Loop { body } => {
                let top = self.builder.reserve();
                let bot = self.builder.reserve();
                let old = self.loopp;
                self.loopp = Some(LoopCtx {
                    bot: *bot,
                    top: *top,
                });
                self.instrs.push(LInstruction::Mark(*top));
                for i in body {
                    self.insert_instr(i);
                }
                self.instrs.push(LInstruction::Jump(*top));
                self.instrs.push(LInstruction::Mark(*bot));
                self.loopp = old;
            }
            UInstruction::Break => {
                self.instrs.push(LInstruction::Jump(
                    self.loopp.expect("Tried to break outside of loop").bot,
                ));
            }
            UInstruction::Continue => {
                self.instrs.push(LInstruction::Jump(
                    self.loopp.expect("Tried to break outside of loop").top,
                ));
            }
        };
        Some(None)
    }

    pub fn finish(self, f: &UFunc) -> IRLFunction {
        IRLFunction {
            instructions: self.instrs,
            makes_call: self.makes_call,
            args: f
                .args
                .iter()
                .map(|a| (*a, self.program.size_of_var(*a).expect("unsized type")))
                .collect(),
            ret_size: self.program.size_of_type(&f.ret).expect("unsized type"),
            stack: self.stack,
            subvar_map: self.subvar_map,
        }
    }
}

impl std::ops::Deref for LProgram {
    type Target = SymbolSpace;

    fn deref(&self) -> &Self::Target {
        &self.sym_space
    }
}
