use std::{collections::HashMap, ops::Deref};

use crate::ir::AddrID;

pub fn create_program<I: Instr>(
    fns: Vec<(Vec<I>, AddrID)>,
    ro_data: Vec<(Vec<u8>, AddrID)>,
    start: Option<AddrID>,
) -> (Vec<u8>, Option<Addr>) {
    let mut data = Vec::new();
    let mut sym_table = SymTable::new(fns.len() + ro_data.len());
    let mut missing = HashMap::<AddrID, Vec<(Addr, I)>>::new();
    for (val, id) in ro_data {
        sym_table.insert(id, Addr(data.len() as u64));
        data.extend(val);
    }
    data.resize(data.len() + (4 - data.len() % 4), 0);
    for (fun, id) in fns {
        sym_table.insert(id, Addr(data.len() as u64));
        for i in fun {
            let i_pos = Addr(data.len() as u64);
            if let Some(sym) = i.push(&mut data, &sym_table, i_pos, false) {
                if let Some(vec) = missing.get_mut(&sym) {
                    vec.push((i_pos, i));
                } else {
                    missing.insert(sym, vec![(i_pos, i)]);
                }
            }
        }
        if let Some(vec) = missing.remove(&id) {
            for (addr, i) in vec {
                let mut replace = Vec::new();
                i.push(&mut replace, &sym_table, addr, true);
                let pos = addr.val() as usize;
                data[pos..pos + replace.len()].copy_from_slice(&replace);
            }
        }
    }
    assert!(missing.is_empty());
    (
        data,
        start.map(|s| sym_table.get(s).expect("start symbol doesn't exist")),
    )
}

pub trait Instr {
    fn push(&self, data: &mut Vec<u8>, syms: &SymTable, pos: Addr, missing: bool)
        -> Option<AddrID>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Addr(u64);
impl Addr {
    const NONE: Self = Self(!0);
    pub fn val(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Symbol(usize);
/// intentionally does not have copy or clone;
/// this should only be consumed once
pub struct WritableSymbol(Symbol);

impl Deref for WritableSymbol {
    type Target = Symbol;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct SymMap<I> {
    i: usize,
    ro_data: Vec<(Vec<u8>, Symbol)>,
    functions: Vec<(Vec<I>, Symbol)>,
}

impl<I> SymMap<I> {
    pub fn new() -> Self {
        Self {
            i: 0,
            ro_data: Vec::new(),
            functions: Vec::new(),
        }
    }
    pub fn push_ro_data(&mut self, data: Vec<u8>) -> Symbol {
        let sym = self.reserve();
        self.write_ro_data(sym, data.into())
    }
    pub fn push_ro_data_size(&mut self, data: Vec<u8>) -> (Symbol, usize) {
        let sym = self.reserve();
        let len = data.len();
        (self.write_ro_data(sym, data), len)
    }
    pub fn push_fn(&mut self, instructions: Vec<I>) -> Symbol {
        let sym = self.reserve();
        self.write_fn(sym, instructions)
    }
    pub fn write_ro_data(&mut self, sym: WritableSymbol, data: Vec<u8>) -> Symbol {
        let data = data.into();
        self.ro_data.push((data, *sym));
        *sym
    }
    pub fn write_fn(&mut self, sym: WritableSymbol, instructions: Vec<I>) -> Symbol {
        self.functions.push((instructions, *sym));
        *sym
    }
    pub fn reserve(&mut self) -> WritableSymbol {
        let val = self.i;
        self.i += 1;
        WritableSymbol(Symbol(val))
    }
    pub fn len(&self) -> usize {
        self.functions.len() + self.ro_data.len()
    }
}

pub struct SymTable(Vec<Addr>);
impl SymTable {
    pub fn new(len: usize) -> Self {
        Self(vec![Addr::NONE; len])
    }
    pub fn insert(&mut self, sym: AddrID, addr: Addr) {
        self.0[sym.0] = addr;
    }
    pub fn get(&self, sym: AddrID) -> Option<Addr> {
        match self.0[sym.0] {
            Addr::NONE => None,
            addr => Some(addr),
        }
    }
}
