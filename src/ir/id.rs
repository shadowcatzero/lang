use std::fmt::Debug;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct TypeID(pub usize);
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct VarID(pub usize);
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct FnID(pub usize);
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct DataID(pub usize);
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct AddrID(pub usize);

impl Debug for VarID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "var{}", self.0)
    }
}

impl Debug for TypeID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ty{}", self.0)
    }
}

impl Debug for FnID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn{}", self.0)
    }
}

impl Debug for DataID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "data{}", self.0)
    }
}

impl Debug for AddrID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}", self.0)
    }
}
