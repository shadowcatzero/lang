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
                            src: src.id,
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
                        let sym = builder.ro_data(src, data);
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
                        let sym = builder.ro_data(src, data);
                        instrs.push(IRLInstruction::LoadAddr {
                            dest: dest.id,
                            offset: 0,
                            src: sym,
                        });

                        let sym = builder.anon_ro_data(&(*len as u64).to_le_bytes());
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
                };
            }
            builder.write_fn(
                sym,
                IRLFunction {
                    name: f.name.clone(),
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
            );
        }
        let sym_space = builder.finish().expect("we failed the mission");
        // println!("fns:");
        // for (a, f) in sym_space.fns() {
        //     println!("    {:?}: {}", a, f.name);
        // }
        // println!("datas: {}", sym_space.ro_data().len());
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
