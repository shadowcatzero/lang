use std::collections::HashMap;

use crate::{
    ir::Symbol,
    util::{Labelable, LabeledFmt},
};

use super::debug::DebugInfo;

pub struct LinkedProgram {
    pub code: Vec<u8>,
    pub start: Option<Addr>,
}

pub struct UnlinkedProgram<I: Instr> {
    pub fns: Vec<UnlinkedFunction<I>>,
    pub ro_data: Vec<(Vec<u8>, Symbol)>,
    pub sym_count: usize,
    pub start: Option<Symbol>,
    pub dbg: DebugInfo,
}

pub struct UnlinkedFunction<I: Instr> {
    pub instrs: Vec<I>,
    pub sym: Symbol,
    pub locations: HashMap<usize, Symbol>,
}

impl<I: Instr + std::fmt::Debug> UnlinkedProgram<I> {
    pub fn link(self) -> LinkedProgram {
        let mut data = Vec::new();
        let mut sym_table = SymTable::new(self.sym_count);
        let mut missing = HashMap::<Symbol, Vec<(Addr, I)>>::new();
        for (val, id) in self.ro_data {
            sym_table.insert(id, Addr(data.len() as u64));
            data.extend(val);
        }
        data.resize(data.len() + (4 - data.len() % 4), 0);
        for f in self.fns {
            let mut added = vec![f.sym];
            sym_table.insert(f.sym, Addr(data.len() as u64));
            for (i, instr) in f.instrs.into_iter().enumerate() {
                let i_pos = Addr(data.len() as u64);
                if let Some(sym) = f.locations.get(&i) {
                    sym_table.insert(*sym, i_pos);
                    added.push(*sym);
                }
                if let Some(sym) = instr.push_to(&mut data, &mut sym_table, i_pos, false) {
                    if let Some(vec) = missing.get_mut(&sym) {
                        vec.push((i_pos, instr));
                    } else {
                        missing.insert(sym, vec![(i_pos, instr)]);
                    }
                }
            }
            for add in added {
                if let Some(vec) = missing.remove(&add) {
                    for (addr, i) in vec {
                        let mut replace = Vec::new();
                        i.push_to(&mut replace, &mut sym_table, addr, true);
                        let pos = addr.val() as usize;
                        data[pos..pos + replace.len()].copy_from_slice(&replace);
                    }
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
    fn push_to(
        &self,
        data: &mut Vec<u8>,
        syms: &mut SymTable,
        pos: Addr,
        missing: bool,
    ) -> Option<Symbol>;
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
        for (fun, irli) in self.fns.iter().zip(&self.dbg.ir_lower) {
            writeln!(f, "{}:", self.dbg.sym_label(fun.sym).unwrap())?;
            let mut liter = irli.iter();
            let mut cur = liter.next();
            for (i, instr) in fun.instrs.iter().enumerate() {
                while let Some(c) = cur
                    && i == c.0
                {
                    writeln!(f, "   {}:", c.1)?;
                    cur = liter.next();
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
