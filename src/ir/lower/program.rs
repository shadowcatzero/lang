use std::collections::HashMap;

use super::{AddrID, IRLData, IRLFunction, IRLInstruction, IRUInstruction, Namespace, VarID};

pub struct IRLProgram {
    pub fns: Vec<IRLFunction>,
    pub data: Vec<IRLData>,
}

// NOTE: there are THREE places here where I specify size (8)

impl IRLProgram {
    pub fn create(ns: &Namespace) -> Self {
        let mut fns = Vec::new();
        let mut data = Vec::new();
        let data_len = ns.data.len();
        for (i, d) in ns.data.iter().enumerate() {
            data.push(IRLData {
                addr: AddrID(i),
                data: d.clone(),
            })
        }
        for (i, f) in ns.fns.iter().enumerate() {
            let f = f.as_ref().unwrap();
            let mut instructions = Vec::new();
            let mut stack = HashMap::new();
            let mut alloc = |i: &VarID| {
                if !stack.contains_key(i) {
                    stack.insert(*i, 8);
                }
            };
            for i in &f.instructions {
                instructions.push(match i {
                    IRUInstruction::Mv { dest, src } => {
                        alloc(dest);
                        IRLInstruction::Mv {
                            dest: *dest,
                            src: *src,
                        }
                    }
                    IRUInstruction::Ref { dest, src } => {
                        alloc(dest);
                        IRLInstruction::Ref {
                            dest: *dest,
                            src: *src,
                        }
                    }
                    IRUInstruction::LoadData { dest, src } => {
                        alloc(dest);
                        IRLInstruction::LoadAddr {
                            dest: *dest,
                            src: AddrID(src.0),
                        }
                    }
                    IRUInstruction::LoadFn { dest, src } => {
                        alloc(dest);
                        IRLInstruction::LoadAddr {
                            dest: *dest,
                            src: AddrID(src.0 + data_len),
                        }
                    }
                    IRUInstruction::Call { dest, f, args } => {
                        alloc(dest);
                        IRLInstruction::Call {
                            dest: *dest,
                            f: AddrID(ns.fn_map[f].0 + data_len),
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
            fns.push(IRLFunction {
                name: f.name.clone(),
                addr: AddrID(i + data_len),
                instructions,
                args: f.args.iter().map(|a| (*a, 8)).collect(),
                stack,
            })
        }
        Self { fns, data }
    }
}
