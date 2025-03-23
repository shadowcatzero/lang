use std::collections::HashMap;

use crate::{
    ir::Symbol,
    util::{Labelable, LabeledFmt, Labeler},
};

use super::debug::DebugInfo;

pub struct LinkedProgram {
    pub code: Vec<u8>,
    pub start: Option<Addr>,
}

pub struct UnlinkedProgram<I: Instr> {
    pub fns: Vec<(Vec<I>, Symbol)>,
    pub ro_data: Vec<(Vec<u8>, Symbol)>,
    pub start: Option<Symbol>,
    pub dbg: DebugInfo,
}

impl<I: Instr> UnlinkedProgram<I> {
    pub fn link(self) -> LinkedProgram {
        let mut data = Vec::new();
        let mut sym_table = SymTable::new(self.fns.len() + self.ro_data.len());
        let mut missing = HashMap::<Symbol, Vec<(Addr, I)>>::new();
        for (val, id) in self.ro_data {
            sym_table.insert(id, Addr(data.len() as u64));
            data.extend(val);
        }
        data.resize(data.len() + (4 - data.len() % 4), 0);
        for (fun, id) in self.fns {
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
        LinkedProgram {
            code: data,
            start: self
                .start
                .map(|s| sym_table.get(s).expect("start symbol doesn't exist")),
        }
    }
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

impl<I: Instr + Labelable<Symbol> + LabeledFmt<Symbol>> std::fmt::Debug for UnlinkedProgram<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for ((v, s), irli) in self.fns.iter().zip(&self.dbg.ir_lower) {
            writeln!(f, "{}:", self.dbg.sym_label(*s).unwrap())?;
            let mut liter = irli.iter();
            let mut cur = liter.next();
            for (i, instr) in v.iter().enumerate() {
                if let Some(c) = cur {
                    if i == c.0 {
                        writeln!(f, "   {}:", c.1)?;
                        cur = liter.next();
                    }
                }
                writeln!(
                    f,
                    "      {:?}",
                    instr.labeled(&|f: &mut std::fmt::Formatter, s: &Symbol| write!(
                        f,
                        "{}",
                        self.dbg.sym_label(*s).unwrap_or(&format!("{:?}", *s))
                    ))
                )?;
            }
        }
        Ok(())
    }
}
