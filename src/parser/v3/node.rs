use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use crate::ir::Origin;

pub struct Node<T> {
    pub inner: Option<T>,
    pub origin: Origin,
}

impl<T> Node<T> {
    pub fn new(inner: T, span: Origin) -> Self {
        Self {
            inner: Some(inner),
            origin: span,
        }
    }
    pub fn bx(self) -> Node<Box<T>> {
        Node {
            inner: self.inner.map(|v| Box::new(v)),
            origin: self.origin,
        }
    }
    pub fn map<T2, F: Fn(T) -> T2>(self, f: F) -> Node<T2> {
        Node {
            inner: self.inner.map(f),
            origin: self.origin,
        }
    }
}

impl<T> Deref for Node<T> {
    type Target = Option<T>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for Node<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: Debug> Debug for Node<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            Some(v) => v.fmt(f),
            None => f.write_str("{error}"),
        }
    }
}
