use std::collections::HashMap;

use crate::ir::{FnID, SymbolSpace};

use super::{IRLFunction, IRLInstruction, IRUInstruction, Len, Namespace, Symbol, VarID};

pub struct IRLProgram {
    sym_space: SymbolSpace,
    entry: Symbol,
}

// NOTE: there are THREE places here where I specify size (8)

impl IRLProgram {
    pub fn create(ns: &Namespace) -> Option<Self> {
        let mut start = None;
        for (i, f) in ns.iter_fns() {
            if f?.name == "start" {
                start = Some(i);
            }
        }
        let start = start?;
        let mut builder = SymbolSpace::with_entries(&[start]);
        let entry = builder.func(&start);
        while let Some((sym, i)) = builder.pop_fn() {
            let f = ns.fns[i.0].as_ref().unwrap();
            let mut instrs = Vec::new();
            let mut stack = HashMap::new();
            let mut makes_call = false;
            let mut alloc_stack = |i: &VarID| -> bool {
                let size = *stack
                    .entry(*i)
                    .or_insert(ns.size_of_var(i).expect("unsized type"));
                size == 0
            };
            for i in &f.instructions {
                match i {
                    IRUInstruction::Mv { dest, src } => {
                        if alloc_stack(dest) {
                            continue;
                        }
                        instrs.push(IRLInstruction::Mv {
                            dest: *dest,
                            src: *src,
                        });
                    }
                    IRUInstruction::Ref { dest, src } => {
                        if alloc_stack(dest) {
                            continue;
                        }
                        instrs.push(IRLInstruction::Ref {
                            dest: *dest,
                            src: *src,
                        });
                    }
                    IRUInstruction::LoadData { dest, src } => {
                        if alloc_stack(dest) {
                            continue;
                        }
                        let data = &ns.data[src.0];
                        let sym = builder.ro_data(src, data);
                        instrs.push(IRLInstruction::LoadData {
                            dest: *dest,
                            offset: 0,
                            len: data.len() as Len,
                            src: sym,
                        });
                    }
                    IRUInstruction::LoadSlice { dest, src, len } => {
                        if alloc_stack(dest) {
                            continue;
                        }
                        let sym = builder.ro_data(src, &ns.data[src.0]);
                        instrs.push(IRLInstruction::LoadAddr {
                            dest: *dest,
                            offset: 0,
                            src: sym,
                        });
                        let sym = builder.anon_ro_data(&(*len as u64).to_le_bytes());
                        instrs.push(IRLInstruction::LoadData {
                            dest: *dest,
                            offset: 8,
                            len: 8,
                            src: sym,
                        });
                    }
                    IRUInstruction::LoadFn { dest, src } => {
                        if alloc_stack(dest) {
                            continue;
                        }
                        let sym = builder.func(src);
                        instrs.push(IRLInstruction::LoadAddr {
                            dest: *dest,
                            offset: 0,
                            src: sym,
                        });
                    }
                    IRUInstruction::Call { dest, f, args } => {
                        alloc_stack(dest);
                        makes_call = true;
                        let fid = &ns.fn_map[f];
                        let sym = builder.func(fid);
                        instrs.push(IRLInstruction::Call {
                            dest: *dest,
                            f: sym,
                            args: args
                                .iter()
                                .map(|a| (*a, ns.size_of_var(a).expect("unsized type")))
                                .collect(),
                        });
                    }
                    IRUInstruction::AsmBlock { instructions, args } => {
                        instrs.push(IRLInstruction::AsmBlock {
                            instructions: instructions.clone(),
                            args: args.clone(),
                        })
                    }
                    IRUInstruction::Ret { src } => instrs.push(IRLInstruction::Ret { src: *src }),
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
                        .map(|a| (*a, ns.size_of_var(a).expect("unsized type")))
                        .collect(),
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
        Some(Self { sym_space, entry })
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
