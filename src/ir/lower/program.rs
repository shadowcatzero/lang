use std::collections::HashMap;

use crate::ir::{IRUFunction, IRUInstrInst, Size, SymbolSpace};

use super::{
    IRLFunction, IRLInstruction, IRUInstruction, IRUProgram, Len, Symbol, SymbolSpaceBuilder, Type,
    VarID,
};

pub struct IRLProgram {
    sym_space: SymbolSpace,
    entry: Symbol,
}

// NOTE: there are THREE places here where I specify size (8)

impl IRLProgram {
    pub fn create(p: &IRUProgram) -> Result<Self, String> {
        let mut start = None;
        for (i, f) in p.iter_fns() {
            if f.name == "start" {
                start = Some(i);
            }
        }
        let start = start.ok_or("no start method found")?;
        let mut ssbuilder = SymbolSpaceBuilder::with_entries(&[start]);
        let entry = ssbuilder.func(&start);
        while let Some((sym, i)) = ssbuilder.pop_fn() {
            let f = p.fns[i.0].as_ref().unwrap();
            let mut fbuilder = IRLFunctionBuilder::new(p, &mut ssbuilder);
            for i in &f.instructions {
                fbuilder.insert_instr(i);
            }
            let res = fbuilder.finish(f);
            ssbuilder.write_fn(sym, res, Some(f.name.clone()));
        }
        let sym_space = ssbuilder.finish().expect("we failed the mission");
        Ok(Self { sym_space, entry })
    }

    pub fn entry(&self) -> Symbol {
        self.entry
    }
}

pub struct IRLFunctionBuilder<'a> {
    program: &'a IRUProgram,
    builder: &'a mut SymbolSpaceBuilder,
    instrs: Vec<IRLInstruction>,
    stack: HashMap<VarID, Size>,
    makes_call: bool,
    outer: Option<Symbol>,
}

impl<'a> IRLFunctionBuilder<'a> {
    pub fn new(program: &'a IRUProgram, builder: &'a mut SymbolSpaceBuilder) -> Self {
        Self {
            instrs: Vec::new(),
            stack: HashMap::new(),
            makes_call: false,
            program,
            builder,
            outer: None,
        }
    }
    pub fn alloc_stack(&mut self, i: VarID) -> Option<()> {
        let size = *self
            .stack
            .entry(i)
            .or_insert(self.program.size_of_var(i).expect("unsized type"));
        if size == 0 {
            None
        } else {
            Some(())
        }
    }
    pub fn insert_instr(&mut self, i: &IRUInstrInst) -> Option<Option<String>> {
        match &i.i {
            IRUInstruction::Mv { dest, src } => {
                self.alloc_stack(dest.id)?;
                self.instrs.push(IRLInstruction::Mv {
                    dest: dest.id,
                    dest_offset: 0,
                    src: src.id,
                    src_offset: 0,
                });
            }
            IRUInstruction::Ref { dest, src } => {
                self.alloc_stack(dest.id)?;
                self.instrs.push(IRLInstruction::Ref {
                    dest: dest.id,
                    src: src.id,
                });
            }
            IRUInstruction::LoadData { dest, src } => {
                self.alloc_stack(dest.id)?;
                let data = &self.program.data[src.0];
                let ddef = self.program.get_data(*src);
                let sym = self.builder.ro_data(src, data, Some(ddef.label.clone()));
                self.instrs.push(IRLInstruction::LoadData {
                    dest: dest.id,
                    offset: 0,
                    len: data.len() as Len,
                    src: sym,
                });
            }
            IRUInstruction::LoadSlice { dest, src } => {
                self.alloc_stack(dest.id)?;
                let data = &self.program.data[src.0];
                let def = self.program.get_data(*src);
                let Type::Array(_, len) = &def.ty else {
                    return Some(Some(format!(
                        "tried to load {} as slice",
                        self.program.type_name(&def.ty)
                    )));
                };
                let sym = self.builder.ro_data(src, data, Some(def.label.clone()));
                self.instrs.push(IRLInstruction::LoadAddr {
                    dest: dest.id,
                    offset: 0,
                    src: sym,
                });

                let sym = self
                    .builder
                    .anon_ro_data(&(*len as u64).to_le_bytes(), Some(format!("len: {}", len)));
                self.instrs.push(IRLInstruction::LoadData {
                    dest: dest.id,
                    offset: 8,
                    len: 8,
                    src: sym,
                });
            }
            IRUInstruction::LoadFn { dest, src } => {
                self.alloc_stack(dest.id)?;
                let sym = self.builder.func(src);
                self.instrs.push(IRLInstruction::LoadAddr {
                    dest: dest.id,
                    offset: 0,
                    src: sym,
                });
            }
            IRUInstruction::Call { dest, f, args } => {
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
                self.instrs.push(IRLInstruction::Call {
                    dest,
                    f: sym,
                    args: args
                        .iter()
                        .map(|a| (a.id, self.program.size_of_var(a.id).expect("unsized type")))
                        .collect(),
                });
            }
            IRUInstruction::AsmBlock { instructions, args } => {
                self.instrs.push(IRLInstruction::AsmBlock {
                    instructions: instructions.clone(),
                    args: args.iter().cloned().map(|(r, v)| (r, v.id)).collect(),
                })
            }
            IRUInstruction::Ret { src } => self.instrs.push(IRLInstruction::Ret { src: src.id }),
            IRUInstruction::Construct { dest, fields } => {
                self.alloc_stack(dest.id)?;
                let ty = &self.program.get_var(dest.id).ty;
                let Type::Concrete(id) = ty else {
                    return Some(Some(format!(
                        "Failed to contruct type {}",
                        self.program.type_name(ty)
                    )));
                };
                let struc = self.program.get_struct(*id);
                for (name, var) in fields {
                    self.instrs.push(IRLInstruction::Mv {
                        dest: dest.id,
                        src: var.id,
                        dest_offset: struc.fields[name].offset,
                        src_offset: 0,
                    })
                }
            }
            IRUInstruction::Access { dest, src, field } => {
                self.alloc_stack(dest.id)?;
                let ty = &self.program.get_var(src.id).ty;
                let Type::Concrete(id) = ty else {
                    return Some(Some(format!(
                        "Failed to access field of struct {}",
                        self.program.type_name(ty)
                    )));
                };
                let struc = self.program.get_struct(*id);
                let Some(field) = struc.fields.get(field) else {
                    return Some(Some(format!(
                        "No field {field} in struct {}",
                        self.program.type_name(ty)
                    )));
                };
                self.instrs.push(IRLInstruction::Mv {
                    dest: dest.id,
                    src: src.id,
                    src_offset: field.offset,
                    dest_offset: 0,
                })
            }
            IRUInstruction::If { cond, body } => {
                let sym = self.builder.reserve();
                self.instrs.push(IRLInstruction::Branch {
                    to: *sym,
                    cond: cond.id,
                });
                for i in body {
                    self.insert_instr(i);
                }
                self.instrs.push(IRLInstruction::Mark(*sym));
            }
            IRUInstruction::Loop { body } => {
                let top = self.builder.reserve();
                let bot = self.builder.reserve();
                let old = self.outer;
                self.outer = Some(*bot);
                self.instrs.push(IRLInstruction::Mark(*top));
                for i in body {
                    self.insert_instr(i);
                }
                self.instrs.push(IRLInstruction::Jump(*top));
                self.instrs.push(IRLInstruction::Mark(*bot));
                self.outer = old;
            }
            IRUInstruction::Break => {
                self.instrs.push(IRLInstruction::Jump(
                    self.outer.expect("Tried to break outside of loop"),
                ));
            }
        };
        Some(None)
    }

    pub fn finish(self, f: &IRUFunction) -> IRLFunction {
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
        }
    }
}

impl std::ops::Deref for IRLProgram {
    type Target = SymbolSpace;

    fn deref(&self) -> &Self::Target {
        &self.sym_space
    }
}
