use std::collections::HashMap;

use super::{
    IRLFunction, LInstruction, Len, Symbol, SymbolSpaceBuilder, UInstruction, UProgram, VarID,
};
use crate::ir::{
    AsmBlockArgType, Size, StructInst, SymbolSpace, Type, TypeID, UFunc, UInstrInst, VarOffset,
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
            let f = &p.fns[i.0];
            let mut fbuilder = LFunctionBuilder::new(p, &mut ssbuilder);
            for i in &f.instructions {
                fbuilder.insert_instr(i);
            }
            if fbuilder.instrs.last().is_none_or(|i| !i.is_ret()) {
                fbuilder.instrs.push(LInstruction::Ret { src: None });
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

pub struct LStructInst {
    offsets: Vec<Len>,
    types: Vec<Type>,
    order: HashMap<String, usize>,
    size: Size,
}

impl LStructInst {
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
    struct_insts: HashMap<StructInst, LStructInst>,
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
        match i
            .i
            .resolve(self.program)
            .expect("failed to resolve during lowering")
        {
            UInstruction::Mv { dst, src } => {
                self.alloc_stack(dst)?;
                self.map_subvar(src);
                self.instrs.push(LInstruction::Mv {
                    dst,
                    dst_offset: 0,
                    src,
                    src_offset: 0,
                });
            }
            UInstruction::Ref { dst, src } => {
                self.alloc_stack(dst)?;
                self.map_subvar(src);
                self.instrs.push(LInstruction::Ref { dst, src });
            }
            UInstruction::Deref { dst, src } => {
                todo!()
            }
            UInstruction::LoadData { dst, src } => {
                self.alloc_stack(dst)?;
                let data = &self.program.data[src];
                let sym = self.data.builder.ro_data(
                    src,
                    &data.content,
                    Some(&self.program.data[src].name),
                );
                self.instrs.push(LInstruction::LoadData {
                    dst,
                    offset: 0,
                    len: data.content.len() as Len,
                    src: sym,
                });
            }
            UInstruction::LoadSlice { dst, src } => {
                self.alloc_stack(dst)?;
                let data = &self.program.data[src];
                let Type::Array(_, len) = &self.program.types[data.ty] else {
                    return Some(Some(format!(
                        "tried to load {} as slice",
                        self.program.type_name(&data.ty)
                    )));
                };
                let sym = self.data.builder.ro_data(
                    src,
                    &data.content,
                    Some(&self.program.data[src].name),
                );
                self.instrs.push(LInstruction::LoadAddr {
                    dst,
                    offset: 0,
                    src: sym,
                });

                let sym = self
                    .builder
                    .anon_ro_data(&(*len as u64).to_le_bytes(), Some(format!("len: {}", len)));
                self.instrs.push(LInstruction::LoadData {
                    dst,
                    offset: 8,
                    len: 8,
                    src: sym,
                });
            }
            UInstruction::Call { dst, f, args } => {
                self.alloc_stack(dst);
                self.makes_call = true;
                let sym = self.builder.func(f.id);
                let ret_size = self
                    .data
                    .size_of_var(self.program, dst)
                    .expect("unsized type");
                let dst = if ret_size > 0 {
                    Some((dst, ret_size))
                } else {
                    None
                };
                let call = LInstruction::Call {
                    dst,
                    f: sym,
                    args: args
                        .into_iter()
                        .map(|id| {
                            self.map_subvar(id);
                            (
                                id,
                                self.data
                                    .size_of_var(self.program, id)
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
                            self.map_subvar(a.var);
                            inputs.push((a.reg, a.var))
                        }
                        AsmBlockArgType::Out => {
                            self.alloc_stack(a.var)?;
                            outputs.push((a.reg, a.var));
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
                self.map_subvar(src);
                let src = if self
                    .data
                    .size_of_var(self.program, src)
                    .expect("unsized var")
                    == 0
                {
                    None
                } else {
                    Some(src)
                };
                self.data.instrs.push(LInstruction::Ret { src })
            }
            UInstruction::Construct {
                dst,
                ref struc,
                ref fields,
            } => {
                self.alloc_stack(dst)?;
                for (field, &src) in fields {
                    self.map_subvar(src);
                    let i = LInstruction::Mv {
                        dst,
                        src,
                        dst_offset: self
                            .data
                            .field_offset(self.program, struc, field)
                            .expect("field offset"),
                        src_offset: 0,
                    };
                    self.instrs.push(i)
                }
            }
            UInstruction::If { cond, body } => {
                self.map_subvar(cond);
                let sym = self.builder.reserve();
                self.instrs.push(LInstruction::Branch { to: *sym, cond });
                for i in body {
                    self.insert_instr(&i);
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
    pub fn struct_inst(&mut self, p: &UProgram, ty: &StructInst) -> &LStructInst {
        // normally I'd let Some(..) here and return, but polonius does not exist :grief:
        if self.struct_insts.get(ty).is_none() {
            let LStructInst { id, args } = ty;
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
                LStructInst {
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

    pub fn size_of_type(&mut self, p: &UProgram, ty: &TypeID) -> Option<Size> {
        // TODO: target matters
        Some(match &p.types[ty] {
            Type::Bits(b) => *b,
            Type::Struct(ty) => self.struct_inst(p, ty).size,
            Type::Generic(id) => return None,
            // function references are resolved at compile time into direct calls,
            // so they don't have any size as arguments
            Type::FnInst(fi) => 0,
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
