use std::fmt::Debug;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct StructID(pub usize);
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct VarID(pub usize);
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct FnID(pub usize);
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct DataID(pub usize);
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct FieldID(pub usize);

// I had an idea for why these were different... now I don't
pub type Size = u32;
pub type Len = u32;

impl Debug for VarID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "var{}", self.0)
    }
}

impl Debug for StructID {
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

impl Debug for FieldID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "field{}", self.0)
    }
}
