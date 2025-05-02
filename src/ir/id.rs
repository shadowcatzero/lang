use std::{
    fmt::Debug,
    marker::PhantomData,
    ops::{Index, IndexMut},
};

// I had an idea for why these were different... now I don't
pub type Size = u32;
pub type Len = u32;

pub struct ID<T>(pub usize, PhantomData<T>);

impl<T> ID<T> {
    pub fn new(i: usize) -> Self {
        Self(i, PhantomData)
    }
}

impl<T> From<usize> for ID<T> {
    fn from(value: usize) -> Self {
        Self(value, PhantomData)
    }
}

impl<T> Debug for ID<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{{}}}", self.0)
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

// :fear:
impl<T> Index<ID<T>> for Vec<T> {
    type Output = T;

    fn index(&self, i: ID<T>) -> &Self::Output {
        &self[i.0]
    }
}

impl<T> IndexMut<ID<T>> for Vec<T> {
    fn index_mut(&mut self, i: ID<T>) -> &mut Self::Output {
        &mut self[i.0]
    }
}

impl<T> Index<&ID<T>> for Vec<T> {
    type Output = T;

    fn index(&self, i: &ID<T>) -> &Self::Output {
        &self[i.0]
    }
}

impl<T> IndexMut<&ID<T>> for Vec<T> {
    fn index_mut(&mut self, i: &ID<T>) -> &mut Self::Output {
        &mut self[i.0]
    }
}

impl<T> Index<ID<T>> for [T] {
    type Output = T;

    fn index(&self, i: ID<T>) -> &Self::Output {
        &self[i.0]
    }
}

impl<T> IndexMut<ID<T>> for [T] {
    fn index_mut(&mut self, i: ID<T>) -> &mut Self::Output {
        &mut self[i.0]
    }
}

impl<T> Index<&ID<T>> for [T] {
    type Output = T;

    fn index(&self, i: &ID<T>) -> &Self::Output {
        &self[i.0]
    }
}

impl<T> IndexMut<&ID<T>> for [T] {
    fn index_mut(&mut self, i: &ID<T>) -> &mut Self::Output {
        &mut self[i.0]
    }
}
