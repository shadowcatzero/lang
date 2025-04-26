use std::collections::HashSet;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Import(pub Vec<String>);
pub type Imports = HashSet<Import>;
