use std::collections::HashMap;

use crate::ir::{AsmBlockArgType, Size, StructTy, SymbolSpace, Type, UFunc, UInstrInst, VarOffset};

use super::{
    IRLFunction, LInstruction, Len, Symbol, SymbolSpaceBuilder, UInstruction, UProgram, VarID,
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
            .id::<UFunc>(&[], "crate")
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
            ssbuilder.write_fn(sym, res, Some(p.names.path(i).to_string()));
        }
        let sym_space = ssbuilder.finish().expect("we failed the mission");
        Ok(Self { sym_space, entry })
    }

    pub fn entry(&self) -> Symbol {
        self.entry
    }
}

pub struct StructInst {
    offsets: Vec<Len>,
    types: Vec<Type>,
    order: HashMap<String, usize>,
    size: Size,
}

impl StructInst {
    pub fn offset(&self, name: &str) -> Option<Len> {
        Some(self.offsets[*self.order.get(name)?])
    }
    pub fn ty(&self, name: &str) -> Option<&Type> {
        Some(&self.types[*self.order.get(name)?])
    }
}

pub struct LFunctionBuilder<'a> {
    data: LFunctionBuilderData<'a>,
    program: &'a UProgram,
}

impl<'a> LFunctionBuilderData<'a> {
    pub fn new(builder: &'a mut SymbolSpaceBuilder) -> Self {
        Self {
            instrs: Vec::new(),
            struct_insts: HashMap::new(),
            stack: HashMap::new(),
            subvar_map: HashMap::new(),
            makes_call: false,
            builder,
            loopp: None,
        }
    }
}

pub struct LFunctionBuilderData<'a> {
    builder: &'a mut SymbolSpaceBuilder,
    instrs: Vec<LInstruction>,
    stack: HashMap<VarID, Size>,
    subvar_map: HashMap<VarID, VarOffset>,
    struct_insts: HashMap<StructInst, StructInst>,
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
            data: LFunctionBuilderData::new(builder),
            program,
        }
    }
    pub fn alloc_stack(&mut self, i: VarID) -> Option<()> {
        if self
            .data
            .size_of_var(self.program, i)
            .expect("unsized type")
            == 0
        {
            return None;
        };
        self.map_subvar(i);
        let var = self.data.var_offset(self.program, i).expect("var offset");
        if !self.stack.contains_key(&var.id) {
            let size = self
                .data
                .size_of_var(self.program, var.id)
                .expect("unsized type");
            self.data.stack.insert(var.id, size);
        }
        Some(())
    }
    pub fn map_subvar(&mut self, i: VarID) {
        let off = self.data.var_offset(self.program, i).expect("var offset");
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
                let sym = self.data.builder.ro_data(
                    src,
                    &data.content,
                    Some(self.program.names.path(dest.id).to_string()),
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
                let sym = self.data.builder.ro_data(
                    src,
                    &data.content,
                    Some(self.program.names.path(dest.id).to_string()),
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
                let fid = &self.program.fn_var.fun(f.id).expect("a");
                let sym = self.builder.func(fid);
                let ret_size = self
                    .data
                    .size_of_var(self.program, dest.id)
                    .expect("unsized type");
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
                            (
                                a.id,
                                self.data
                                    .size_of_var(self.program, a.id)
                                    .expect("unsized type"),
                            )
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
                let src = if self
                    .data
                    .size_of_var(self.program, src.id)
                    .expect("unsized var")
                    == 0
                {
                    None
                } else {
                    Some(src.id)
                };
                self.data.instrs.push(LInstruction::Ret { src })
            }
            UInstruction::Construct { dest, fields } => {
                let sty = &self.program.expect_type(dest.id);
                let Type::Struct(sty) = sty else {
                    panic!("bruh htis aint' a struct");
                };
                self.alloc_stack(dest.id)?;
                for (field, var) in fields {
                    self.map_subvar(var.id);
                    let i = LInstruction::Mv {
                        dest: dest.id,
                        src: var.id,
                        dest_offset: self
                            .data
                            .field_offset(self.program, sty, field)
                            .expect("field offset"),
                        src_offset: 0,
                    };
                    self.instrs.push(i)
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
                self.data.instrs.push(LInstruction::Jump(
                    self.data.loopp.expect("Tried to break outside of loop").bot,
                ));
            }
            UInstruction::Continue => {
                self.data.instrs.push(LInstruction::Jump(
                    self.data.loopp.expect("Tried to break outside of loop").top,
                ));
            }
        };
        Some(None)
    }

    pub fn finish(mut self, f: &UFunc) -> IRLFunction {
        IRLFunction {
            args: f
                .args
                .iter()
                .map(|a| {
                    (
                        *a,
                        self.data
                            .size_of_var(self.program, *a)
                            .expect("unsized type"),
                    )
                })
                .collect(),
            ret_size: self
                .data
                .size_of_type(self.program, &f.ret)
                .expect("unsized type"),
            instructions: self.data.instrs,
            makes_call: self.data.makes_call,
            stack: self.data.stack,
            subvar_map: self.data.subvar_map,
        }
    }
}

impl LFunctionBuilderData<'_> {
    pub fn var_offset(&mut self, p: &UProgram, mut var: VarID) -> Option<VarOffset> {
        let mut path = Vec::new();
        while let Type::Field(parent) = &p.get(var)?.ty {
            var = parent.parent;
            path.push(&parent.name);
        }
        let mut ty = &p.get(var)?.ty;
        let mut offset = 0;
        while let Type::Struct(sty) = ty {
            let Some(name) = path.pop() else {
                break;
            };
            offset += self.field_offset(p, sty, &name)?;
            ty = p.struct_field_type(sty, name).expect("bad field");
        }
        Some(VarOffset { id: var, offset })
    }
    pub fn addr_size(&self) -> Size {
        64
    }
    pub fn struct_inst(&mut self, p: &UProgram, ty: &StructTy) -> &StructInst {
        // normally I'd let Some(..) here and return, but polonius does not exist :grief:
        if self.struct_insts.get(ty).is_none() {
            let StructInst { id, args } = ty;
            let struc = p.expect(*id);
            let mut types = Vec::new();
            let mut sizes = struc
                .fields
                .iter()
                .map(|(n, f)| {
                    let ty = if let Type::Generic { id } = &f.ty {
                        struc
                            .generics
                            .iter()
                            .enumerate()
                            .find_map(|(i, g)| if *g == *id { args.get(i) } else { None })
                            .unwrap_or(&f.ty)
                    } else {
                        &f.ty
                    };
                    types.push(ty.clone());
                    (n, self.size_of_type(p, ty).expect("unsized type"))
                })
                .collect::<Vec<_>>();
            sizes.sort_by(|(n1, s1, ..), (n2, s2, ..)| s1.cmp(s2).then_with(|| n1.cmp(n2)));
            let mut offset = 0;
            let mut offsets = Vec::new();
            let mut order = HashMap::new();
            for (i, (name, size)) in sizes.iter().rev().enumerate() {
                // TODO: alignment!!!
                order.insert(name.to_string(), i);
                offsets.push(offset);
                offset += size;
            }
            self.struct_insts.insert(
                ty.clone(),
                StructInst {
                    offsets,
                    order,
                    types,
                    size: offset,
                },
            );
        }
        self.struct_insts.get(ty).unwrap()
    }

    pub fn field_offset(&mut self, p: &UProgram, sty: &StructInst, field: &str) -> Option<Len> {
        let inst = self.struct_inst(p, sty);
        Some(inst.offset(field)?)
    }

    pub fn size_of_type(&mut self, p: &UProgram, ty: &Type) -> Option<Size> {
        // TODO: target matters
        Some(match p.follow_type(ty)? {
            Type::Bits(b) => *b,
            Type::Struct(ty) => self.struct_inst(p, ty).size,
            Type::Generic { id } => return None,
            Type::Fn { args, ret } => todo!(),
            Type::Ref(_) => self.addr_size(),
            Type::Array(ty, len) => self.size_of_type(p, ty)? * len,
            Type::Slice(_) => self.addr_size() * 2,
            Type::Unit => 0,
            _ => return None,
        })
    }

    pub fn size_of_var(&mut self, p: &UProgram, var: VarID) -> Option<Size> {
        self.size_of_type(p, &p.get(var)?.ty)
    }
}

impl<'a> std::ops::Deref for LFunctionBuilder<'a> {
    type Target = LFunctionBuilderData<'a>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a> std::ops::DerefMut for LFunctionBuilder<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl std::ops::Deref for LProgram {
    type Target = SymbolSpace;

    fn deref(&self) -> &Self::Target {
        &self.sym_space
    }
}
