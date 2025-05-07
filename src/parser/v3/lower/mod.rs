mod arch;
mod asm;
mod block;
mod def;
mod expr;
mod func;
mod map;
mod struc;
mod ty;

use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use super::*;
use crate::{
    ir::{
        IdentID, IdentStatus, ModID, Origin, Res, Type, TypeID, UFunc, UIdent, UModule, UProgram,
        UVar,
    },
    util::NameStack,
};
pub use func::{FnLowerCtx, FnLowerable};

impl PModule {
    pub fn lower(
        &self,
        path: Vec<String>,
        p: &mut UProgram,
        imports: &mut Imports,
        output: &mut CompilerOutput,
    ) -> ModID {
        let name = path.last().unwrap().clone();
        let f = UFunc {
            name: name.clone(),
            args: Vec::new(),
            instructions: Vec::new(),
            gargs: Vec::new(),
            ret: p.def_ty(Type::Unit),
            origin: self.block.origin,
        };
        let fid = p.def_fn(f);
        let mid = p.def_module(UModule {
            name,
            members: HashMap::new(),
            parent: None,
            func: fid,
        });
        let mut ctx = ModuleLowerCtx {
            p,
            output,
            module: mid,
            temp: 0,
            ident_stack: NameStack::new(),
        };
        let mut fctx = FnLowerCtx {
            ctx: &mut ctx,
            instructions: Vec::new(),
            origin: self.block.origin,
        };
        self.block.lower(&mut fctx);
        p.fns[fid].instructions = fctx.instructions;
        mid
    }
}

pub struct ModuleLowerCtx<'a> {
    pub p: &'a mut UProgram,
    pub output: &'a mut CompilerOutput,
    pub module: ModID,
    pub temp: usize,
    pub ident_stack: NameStack<IdentID>,
}

impl<'a> ModuleLowerCtx<'a> {
    pub fn new(program: &'a mut UProgram, output: &'a mut CompilerOutput, id: ModID) -> Self {
        Self {
            p: program,
            output,
            module: id,
            temp: 0,
            ident_stack: NameStack::new(),
        }
    }
    pub fn temp_var(&mut self, origin: Origin, ty: impl Typable) -> IdentID {
        self.temp_var_inner(origin, ty)
    }
    fn temp_var_inner(&mut self, origin: Origin, ty: impl Typable) -> IdentID {
        let var = UVar {
            name: format!("temp{}", self.temp),
            ty: ty.ty(self),
            origin,
            parent: None,
            children: HashMap::new(),
        };
        let id = self.p.def_var(var);
        self.temp += 1;
        self.def_ident(UIdent {
            status: IdentStatus::Res(Res::Var(id)),
            origin,
        })
    }
}

pub trait Typable {
    fn ty(self, p: &mut UProgram) -> TypeID;
}

impl Typable for Type {
    fn ty(self, p: &mut UProgram) -> TypeID {
        p.def_ty(self)
    }
}

impl Typable for TypeID {
    fn ty(self, p: &mut UProgram) -> TypeID {
        self
    }
}

impl Deref for ModuleLowerCtx<'_> {
    type Target = UProgram;

    fn deref(&self) -> &Self::Target {
        self.p
    }
}

impl DerefMut for ModuleLowerCtx<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.p
    }
}
