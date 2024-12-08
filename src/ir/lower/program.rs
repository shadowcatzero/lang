use std::collections::HashMap;

use crate::ir::{FnID, SymbolSpace};

use super::{IRLFunction, IRLInstruction, IRUInstruction, Namespace, Symbol, VarID};

pub struct IRLProgram {
    sym_space: SymbolSpace,
    entry: Symbol,
}

// NOTE: there are THREE places here where I specify size (8)

impl IRLProgram {
    pub fn create(ns: &Namespace) -> Option<Self> {
        let mut start = None;
        for (i, f) in ns.fns.iter().enumerate() {
            let f = f.as_ref()?;
            if f.name == "start" {
                start = Some(FnID(i));
            }
        }
        let start = start?;
        let mut builder = SymbolSpace::with_entries(&[start]);
        let entry = builder.func(&start);
        while let Some((sym, i)) = builder.pop_fn() {
            let f = ns.fns[i.0].as_ref().unwrap();
            let mut instructions = Vec::new();
            let mut stack = HashMap::new();
            let mut alloc_stack = |i: &VarID| {
                if !stack.contains_key(i) {
                    stack.insert(*i, 8);
                }
            };
            for i in &f.instructions {
                instructions.push(match i {
                    IRUInstruction::Mv { dest, src } => {
                        alloc_stack(dest);
                        IRLInstruction::Mv {
                            dest: *dest,
                            src: *src,
                        }
                    }
                    IRUInstruction::Ref { dest, src } => {
                        alloc_stack(dest);
                        IRLInstruction::Ref {
                            dest: *dest,
                            src: *src,
                        }
                    }
                    IRUInstruction::LoadData { dest, src } => {
                        alloc_stack(dest);
                        let addr = builder.ro_data(src, &ns.data[src.0]);
                        IRLInstruction::LoadAddr {
                            dest: *dest,
                            src: addr,
                        }
                    }
                    IRUInstruction::LoadFn { dest, src } => {
                        alloc_stack(dest);
                        let sym = builder.func(src);
                        IRLInstruction::LoadAddr {
                            dest: *dest,
                            src: sym,
                        }
                    }
                    IRUInstruction::Call { dest, f, args } => {
                        alloc_stack(dest);
                        let fid = &ns.fn_map[f];
                        let sym = builder.func(fid);
                        IRLInstruction::Call {
                            dest: *dest,
                            f: sym,
                            args: args.iter().map(|a| (*a, 8)).collect(),
                        }
                    }
                    IRUInstruction::AsmBlock { instructions, args } => IRLInstruction::AsmBlock {
                        instructions: instructions.clone(),
                        args: args.clone(),
                    },
                    IRUInstruction::Ret { src } => IRLInstruction::Ret { src: *src },
                });
            }
            builder.write_fn(
                sym,
                IRLFunction {
                    name: f.name.clone(),
                    instructions,
                    args: f.args.iter().map(|a| (*a, 8)).collect(),
                    stack,
                },
            );
        }
        let sym_space = builder.finish().expect("we failed the mission");
        println!("fns:");
        for (a, f) in sym_space.fns() {
            println!("    {:?}: {}", a, f.name);
        }
        println!("datas: {}", sym_space.ro_data().len());
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
