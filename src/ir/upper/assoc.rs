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

pub type NamePath = Vec<String>;

pub struct NameTree {
    ids: [HashMap<String, usize>; NAMED_KINDS],
    children: HashMap<String, NameTree>,
}

impl NameTree {
    pub fn new() -> Self {
        Self {
            ids: core::array::from_fn(|_| HashMap::new()),
            children: HashMap::new(),
        }
    }
    pub fn get(&self, path: &[String]) -> Option<&NameTree> {
        let first = path.first()?;
        self.children.get(first)?.get(&path[1..])
    }
    pub fn id<K: Kind>(&self, path: &[String]) -> Option<ID<K>> {
        let last = path.last()?;
        self.get(&path[..path.len() - 1])?.ids[K::INDEX]
            .get(last)
            .copied()
            .map(ID::new)
    }
    pub fn insert<K: Kind>(&mut self, path: &[String], id: usize) {
        if let [key] = &path[..] {
            self.ids[K::INDEX].insert(key.to_string(), id);
            return;
        }
        let Some(key) = path.first() else {
            return;
        };
        self.children
            .entry(key.to_string())
            .or_insert_with(|| NameTree::new())
            .insert::<K>(&path[1..], id);
    }
}

pub struct NameMap {
    names: [Vec<String>; NAMED_KINDS],
    tree: NameTree,
}

impl NameMap {
    pub fn new() -> Self {
        Self {
            names: core::array::from_fn(|_| Vec::new()),
            tree: NameTree::new(),
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
    pub fn id<K: Kind>(&self, path: &[String]) -> Option<ID<K>> {
        Some(self.tree.id(path)?)
    }
    pub fn push<K: Kind>(&mut self, path: &[String]) {
        let id = self.names[K::INDEX].len();
        self.tree.insert::<K>(path, id);
        let name = path.join("::");
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
