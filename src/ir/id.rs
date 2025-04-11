use std::{fmt::Debug, marker::PhantomData};

// I had an idea for why these were different... now I don't
pub type Size = u32;
pub type Len = u32;

pub struct ID<T>(pub usize, PhantomData<T>);

impl<T> ID<T> {
    pub fn new(i: usize) -> Self {
        Self(i, PhantomData)
    }
}

pub trait Named {
    const NAME: &str;
}

impl<T> From<usize> for ID<T> {
    fn from(value: usize) -> Self {
        Self(value, PhantomData)
    }
}

impl<K: Named> Debug for ID<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", K::NAME, self.0)
    }
}

impl<T> PartialEq for ID<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for ID<T> {}

impl<T> std::hash::Hash for ID<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T> Clone for ID<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<T> Copy for ID<T> {}
