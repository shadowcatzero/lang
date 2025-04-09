use std::collections::HashMap;

use crate::ir::{AsmBlockArgType, IRUFunction, IRUInstrInst, Size, SymbolSpace, VarOffset};

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
            if fbuilder.instrs.last().is_none_or(|i| !i.is_ret()) {
                fbuilder.instrs.push(IRLInstruction::Ret { src: None });
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
    subvar_map: HashMap<VarID, VarOffset>,
    makes_call: bool,
    loopp: Option<LoopCtx>,
}

#[derive(Clone, Copy)]
pub struct LoopCtx {
    top: Symbol,
    bot: Symbol,
}

impl<'a> IRLFunctionBuilder<'a> {
    pub fn new(program: &'a IRUProgram, builder: &'a mut SymbolSpaceBuilder) -> Self {
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
    pub fn insert_instr(&mut self, i: &IRUInstrInst) -> Option<Option<String>> {
        match &i.i {
            IRUInstruction::Mv { dest, src } => {
                self.alloc_stack(dest.id)?;
                self.map_subvar(src.id);
                self.instrs.push(IRLInstruction::Mv {
                    dest: dest.id,
                    dest_offset: 0,
                    src: src.id,
                    src_offset: 0,
                });
            }
            IRUInstruction::Ref { dest, src } => {
                self.alloc_stack(dest.id)?;
                self.map_subvar(src.id);
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
                let call = IRLInstruction::Call {
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
            IRUInstruction::AsmBlock { instructions, args } => {
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
                self.instrs.push(IRLInstruction::AsmBlock {
                    instructions: instructions.clone(),
                    inputs,
                    outputs,
                })
            }
            IRUInstruction::Ret { src } => {
                self.map_subvar(src.id);
                self.instrs.push(IRLInstruction::Ret {
                    src: if self.program.size_of_var(src.id).expect("unsized var") == 0 {
                        None
                    } else {
                        Some(src.id)
                    },
                })
            }
            IRUInstruction::Construct { dest, fields } => {
                self.alloc_stack(dest.id)?;
                let ty = &self.program.get_var(dest.id).ty;
                let &Type::Struct { id, ref args } = ty else {
                    return Some(Some(format!(
                        "Failed to contruct type {}",
                        self.program.type_name(ty)
                    )));
                };
                for (&fid, var) in fields {
                    self.map_subvar(var.id);
                    self.instrs.push(IRLInstruction::Mv {
                        dest: dest.id,
                        src: var.id,
                        dest_offset: self.program.field_offset(id, fid).expect("field offset"),
                        src_offset: 0,
                    })
                }
            }
            IRUInstruction::If { cond, body } => {
                self.map_subvar(cond.id);
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
                let old = self.loopp;
                self.loopp = Some(LoopCtx {
                    bot: *bot,
                    top: *top,
                });
                self.instrs.push(IRLInstruction::Mark(*top));
                for i in body {
                    self.insert_instr(i);
                }
                self.instrs.push(IRLInstruction::Jump(*top));
                self.instrs.push(IRLInstruction::Mark(*bot));
                self.loopp = old;
            }
            IRUInstruction::Break => {
                self.instrs.push(IRLInstruction::Jump(
                    self.loopp.expect("Tried to break outside of loop").bot,
                ));
            }
            IRUInstruction::Continue => {
                self.instrs.push(IRLInstruction::Jump(
                    self.loopp.expect("Tried to break outside of loop").top,
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
            subvar_map: self.subvar_map,
        }
    }
}

impl std::ops::Deref for IRLProgram {
    type Target = SymbolSpace;

    fn deref(&self) -> &Self::Target {
        &self.sym_space
    }
}
