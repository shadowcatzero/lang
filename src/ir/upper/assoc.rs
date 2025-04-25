use super::{FnID, Kind, Origin, VarID, NAMED_KINDS};
use crate::ir::ID;
use std::collections::HashMap;

pub struct OriginMap {
    origins: [Vec<Origin>; NAMED_KINDS],
}

impl OriginMap {
    pub fn new() -> Self {
        Self {
            origins: core::array::from_fn(|_| Vec::new()),
        }
    }
    pub fn get<K: Kind>(&self, id: ID<K>) -> Origin {
        self.origins[K::INDEX][id.0]
    }
    pub fn push<K: Kind>(&mut self, origin: Origin) {
        self.origins[K::INDEX].push(origin);
    }
}

pub struct NameMap {
    names: [Vec<String>; NAMED_KINDS],
    inv_names: [HashMap<String, usize>; NAMED_KINDS],
}

impl NameMap {
    pub fn new() -> Self {
        Self {
            names: core::array::from_fn(|_| Vec::new()),
            inv_names: core::array::from_fn(|_| HashMap::new()),
        }
    }
    pub fn path<K: Kind>(&self, id: ID<K>) -> &str {
        &self.names[K::INDEX][id.0]
    }
    pub fn name<K: Kind>(&self, id: ID<K>) -> &str {
        let mut path = self.path(id);
        while let Some(i) = path.find("::") {
            path = &path[i + 2..];
        }
        path
    }
    pub fn id<K: Kind>(&self, name: &str) -> Option<ID<K>> {
        Some(ID::new(*self.inv_names[K::INDEX].get(name)?))
    }
    pub fn push<K: Kind>(&mut self, name: String) {
        self.inv_names[K::INDEX].insert(name.clone(), self.names[K::INDEX].len());
        self.names[K::INDEX].push(name);
    }
}

pub struct FnVarMap {
    vtf: HashMap<VarID, FnID>,
    ftv: Vec<VarID>,
}

impl FnVarMap {
    pub fn new() -> Self {
        Self {
            vtf: HashMap::new(),
            ftv: Vec::new(),
        }
    }
    pub fn insert(&mut self, f: FnID, v: VarID) {
        self.vtf.insert(v, f);
        self.ftv.push(v);
    }
    pub fn var(&self, f: FnID) -> VarID {
        self.ftv[f.0]
    }
    pub fn fun(&self, v: VarID) -> Option<FnID> {
        self.vtf.get(&v).copied()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Ident {
    id: usize,
    kind: usize,
}

impl<K: Kind> From<ID<K>> for Ident {
    fn from(id: ID<K>) -> Self {
        Self {
            id: id.0,
            kind: K::INDEX,
        }
    }
}

// this isn't really a map... but also keeps track of "side data"
#[derive(Debug, Clone, Copy)]
pub struct Idents {
    pub latest: Ident,
    pub kinds: [Option<usize>; NAMED_KINDS],
}

impl Idents {
    pub fn new(latest: Ident) -> Self {
        let mut s = Self {
            latest,
            kinds: [None; NAMED_KINDS],
        };
        s.insert(latest);
        s
    }
    pub fn insert(&mut self, i: Ident) {
        self.latest = i;
        self.kinds[i.kind] = Some(i.id);
    }
    pub fn get<K: Kind>(&self) -> Option<ID<K>> {
        self.kinds[K::INDEX].map(|i| i.into())
    }
}
