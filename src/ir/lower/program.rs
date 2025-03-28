use std::collections::HashMap;

use crate::ir::SymbolSpace;

use super::{IRLFunction, IRLInstruction, IRUInstruction, IRUProgram, Len, Symbol, Type, VarID};

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
        let mut builder = SymbolSpace::with_entries(&[start]);
        let entry = builder.func(&start);
        while let Some((sym, i)) = builder.pop_fn() {
            let f = p.fns[i.0].as_ref().unwrap();
            let mut instrs = Vec::new();
            let mut stack = HashMap::new();
            let mut makes_call = false;
            let mut alloc_stack = |i: VarID| -> bool {
                let size = *stack
                    .entry(i)
                    .or_insert(p.size_of_var(i).expect("unsized type"));
                size == 0
            };
            for i in &f.instructions {
                match &i.i {
                    IRUInstruction::Mv { dest, src } => {
                        if alloc_stack(dest.id) {
                            continue;
                        }
                        instrs.push(IRLInstruction::Mv {
                            dest: dest.id,
                            dest_offset: 0,
                            src: src.id,
                            src_offset: 0,
                        });
                    }
                    IRUInstruction::Ref { dest, src } => {
                        if alloc_stack(dest.id) {
                            continue;
                        }
                        instrs.push(IRLInstruction::Ref {
                            dest: dest.id,
                            src: src.id,
                        });
                    }
                    IRUInstruction::LoadData { dest, src } => {
                        if alloc_stack(dest.id) {
                            continue;
                        }
                        let data = &p.data[src.0];
                        let ddef = p.get_data(*src);
                        let sym = builder.ro_data(src, data, Some(ddef.label.clone()));
                        instrs.push(IRLInstruction::LoadData {
                            dest: dest.id,
                            offset: 0,
                            len: data.len() as Len,
                            src: sym,
                        });
                    }
                    IRUInstruction::LoadSlice { dest, src } => {
                        if alloc_stack(dest.id) {
                            continue;
                        }
                        let data = &p.data[src.0];
                        let def = p.get_data(*src);
                        let Type::Array(ty, len) = &def.ty else {
                            return Err(format!("tried to load {} as slice", p.type_name(&def.ty)));
                        };
                        let sym = builder.ro_data(src, data, Some(def.label.clone()));
                        instrs.push(IRLInstruction::LoadAddr {
                            dest: dest.id,
                            offset: 0,
                            src: sym,
                        });

                        let sym = builder.anon_ro_data(
                            &(*len as u64).to_le_bytes(),
                            Some(format!("len: {}", len)),
                        );
                        instrs.push(IRLInstruction::LoadData {
                            dest: dest.id,
                            offset: 8,
                            len: 8,
                            src: sym,
                        });
                    }
                    IRUInstruction::LoadFn { dest, src } => {
                        if alloc_stack(dest.id) {
                            continue;
                        }
                        let sym = builder.func(src);
                        instrs.push(IRLInstruction::LoadAddr {
                            dest: dest.id,
                            offset: 0,
                            src: sym,
                        });
                    }
                    IRUInstruction::Call { dest, f, args } => {
                        alloc_stack(dest.id);
                        makes_call = true;
                        let fid = &p.fn_map[&f.id];
                        let sym = builder.func(fid);
                        let ret_size = p.size_of_var(dest.id).expect("unsized type");
                        let dest = if ret_size > 0 {
                            Some((dest.id, ret_size))
                        } else {
                            None
                        };
                        instrs.push(IRLInstruction::Call {
                            dest,
                            f: sym,
                            args: args
                                .iter()
                                .map(|a| (a.id, p.size_of_var(a.id).expect("unsized type")))
                                .collect(),
                        });
                    }
                    IRUInstruction::AsmBlock { instructions, args } => {
                        instrs.push(IRLInstruction::AsmBlock {
                            instructions: instructions.clone(),
                            args: args.iter().cloned().map(|(r, v)| (r, v.id)).collect(),
                        })
                    }
                    IRUInstruction::Ret { src } => instrs.push(IRLInstruction::Ret { src: src.id }),
                    IRUInstruction::Construct { dest, fields } => {
                        if alloc_stack(dest.id) {
                            continue;
                        }
                        let ty = &p.get_var(dest.id).ty;
                        let Type::Concrete(id) = ty else {
                            return Err(format!("Failed to contruct type {}", p.type_name(ty)));
                        };
                        let struc = p.get_struct(*id);
                        for (name, var) in fields {
                            instrs.push(IRLInstruction::Mv {
                                dest: dest.id,
                                src: var.id,
                                dest_offset: struc.fields[name].offset,
                                src_offset: 0,
                            })
                        }
                    }
                    IRUInstruction::Access { dest, src, field } => {
                        if alloc_stack(dest.id) {
                            continue;
                        }
                        let ty = &p.get_var(src.id).ty;
                        let Type::Concrete(id) = ty else {
                            return Err(format!(
                                "Failed to access field of struct {}",
                                p.type_name(ty)
                            ));
                        };
                        let struc = p.get_struct(*id);
                        let Some(field) = struc.fields.get(field) else {
                            return Err(format!("No field {field} in struct {}", p.type_name(ty)));
                        };
                        instrs.push(IRLInstruction::Mv {
                            dest: dest.id,
                            src: src.id,
                            src_offset: field.offset,
                            dest_offset: 0,
                        })
                    }
                };
            }
            builder.write_fn(
                sym,
                IRLFunction {
                    instructions: instrs,
                    makes_call,
                    args: f
                        .args
                        .iter()
                        .map(|a| (*a, p.size_of_var(*a).expect("unsized type")))
                        .collect(),
                    ret_size: p.size_of_type(&f.ret).expect("unsized type"),
                    stack,
                },
                Some(f.name.clone()),
            );
        }
        let sym_space = builder.finish().expect("we failed the mission");
        Ok(Self { sym_space, entry })
    }

    pub fn entry(&self) -> Symbol {
        self.entry
    }
}

impl std::ops::Deref for IRLProgram {
    type Target = SymbolSpace;

    fn deref(&self) -> &Self::Target {
        &self.sym_space
    }
}
