use std::collections::HashMap;

use super::{DataID, FnID, IRLFunction};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Symbol(usize);
/// intentionally does not have copy or clone;
/// this should only be consumed once
pub struct WritableSymbol(Symbol);

impl std::ops::Deref for WritableSymbol {
    type Target = Symbol;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct SymbolSpace {
    ro_data: Vec<(Symbol, Vec<u8>)>,
    fns: Vec<(Symbol, IRLFunction)>,
    len: usize,
    labels: Vec<Option<String>>,
}

pub struct SymbolSpaceBuilder {
    symbols: usize,
    unwritten_fns: Vec<(WritableSymbol, FnID)>,
    fn_map: HashMap<FnID, Symbol>,
    data_map: HashMap<DataID, Symbol>,
    ro_data: Vec<(Symbol, Vec<u8>)>,
    fns: Vec<(Symbol, IRLFunction)>,
    labels: Vec<Option<String>>,
}

impl SymbolSpace {
    pub fn ro_data(&self) -> &[(Symbol, Vec<u8>)] {
        &self.ro_data
    }
    pub fn fns(&self) -> &[(Symbol, IRLFunction)] {
        &self.fns
    }
    pub fn labels(&self) -> &[Option<String>] {
        &self.labels
    }
    pub fn len(&self) -> usize {
        self.len
    }
}

impl SymbolSpaceBuilder {
    pub fn new() -> Self {
        Self {
            symbols: 0,
            unwritten_fns: Vec::new(),
            fn_map: HashMap::new(),
            data_map: HashMap::new(),
            ro_data: Vec::new(),
            fns: Vec::new(),
            labels: Vec::new(),
        }
    }
    pub fn with_entries(entries: &[FnID]) -> SymbolSpaceBuilder {
        let mut s = Self::new();
        for e in entries {
            s.func(e);
        }
        s
    }
    pub fn pop_fn(&mut self) -> Option<(WritableSymbol, FnID)> {
        self.unwritten_fns.pop()
    }
    pub fn anon_ro_data(&mut self, data: &[u8], label: Option<String>) -> Symbol {
        let sym = self.reserve();
        self.write_ro_data(sym, data.to_vec(), label)
    }
    pub fn ro_data(&mut self, id: &DataID, data: &[u8], label: Option<String>) -> Symbol {
        match self.data_map.get(id) {
            Some(s) => *s,
            None => {
                let sym = self.reserve();
                self.data_map.insert(*id, *sym);
                self.write_ro_data(sym, data.to_vec(), label)
            }
        }
    }
    pub fn func(&mut self, id: &FnID) -> Symbol {
        match self.fn_map.get(id) {
            Some(s) => *s,
            None => {
                let wsym = self.reserve();
                let sym = *wsym;
                self.unwritten_fns.push((wsym, *id));
                self.fn_map.insert(*id, sym);
                sym
            }
        }
    }
    pub fn write_ro_data(
        &mut self,
        sym: WritableSymbol,
        data: Vec<u8>,
        name: Option<String>,
    ) -> Symbol {
        self.ro_data.push((*sym, data));
        self.labels[sym.0 .0] = name;
        *sym
    }
    pub fn write_fn(
        &mut self,
        sym: WritableSymbol,
        func: IRLFunction,
        name: Option<String>,
    ) -> Symbol {
        self.fns.push((*sym, func));
        self.labels[sym.0 .0] = name;
        *sym
    }
    pub fn reserve(&mut self) -> WritableSymbol {
        let val = self.symbols;
        self.symbols += 1;
        self.labels.push(None);
        WritableSymbol(Symbol(val))
    }
    pub fn len(&self) -> usize {
        self.symbols
    }
    pub fn finish(self) -> Option<SymbolSpace> {
        if self.unwritten_fns.is_empty() {
            Some(SymbolSpace {
                len: self.symbols,
                fns: self.fns,
                ro_data: self.ro_data,
                labels: self.labels,
            })
        } else {
            None
        }
    }
}

impl std::fmt::Debug for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}", self.0)
    }
}

impl std::ops::Deref for Symbol {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
