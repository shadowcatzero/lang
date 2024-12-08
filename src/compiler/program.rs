use std::collections::HashMap;

use crate::ir::Symbol;

pub fn create_program<I: Instr>(
    fns: Vec<(Vec<I>, Symbol)>,
    ro_data: Vec<(Vec<u8>, Symbol)>,
    start: Option<Symbol>,
) -> (Vec<u8>, Option<Addr>) {
    let mut data = Vec::new();
    let mut sym_table = SymTable::new(fns.len() + ro_data.len());
    let mut missing = HashMap::<Symbol, Vec<(Addr, I)>>::new();
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
        -> Option<Symbol>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Addr(u64);
impl Addr {
    const NONE: Self = Self(!0);
    pub fn val(&self) -> u64 {
        self.0
    }
}

pub struct SymTable(Vec<Addr>);
impl SymTable {
    pub fn new(len: usize) -> Self {
        Self(vec![Addr::NONE; len])
    }
    pub fn insert(&mut self, sym: Symbol, addr: Addr) {
        self.0[*sym] = addr;
    }
    pub fn get(&self, sym: Symbol) -> Option<Addr> {
        match self.0[*sym] {
            Addr::NONE => None,
            addr => Some(addr),
        }
    }
}
