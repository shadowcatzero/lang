use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use super::FileSpan;

pub trait MaybeResolved {
    type Inner<T>;
}

pub struct Resolved;
impl MaybeResolved for Resolved {
    type Inner<T> = T;
}

pub struct Unresolved;
impl MaybeResolved for Unresolved {
    type Inner<T> = Result<T, ()>;
}

pub struct Node<T, R: MaybeResolved> {
    pub inner: <R as MaybeResolved>::Inner<T>,
    pub span: FileSpan,
}

impl<T> Node<T, Unresolved> {
    pub fn new(inner: T, span: FileSpan) -> Self {
        Self {
            inner: Ok(inner),
            span,
        }
    }
    pub fn bx(self) -> Node<Box<T>, Unresolved> {
        Node {
            inner: self.inner.map(|v| Box::new(v)),
            span: self.span,
        }
    }
}

impl<T, R: MaybeResolved> Deref for Node<T, R> {
    type Target = <R as MaybeResolved>::Inner<T>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T, R: MaybeResolved> DerefMut for Node<T, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: Debug> Debug for Node<T, Unresolved> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            Ok(v) => v.fmt(f),
            Err(_) => f.write_str("{error}"),
        }
    }
}

impl<T: Debug> Debug for Node<T, Resolved> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

pub trait Resolvable<Res> {
    fn resolve(self) -> Result<Res, ()>;
}

impl<T: Resolvable<Res>, Res> Resolvable<Node<Res, Resolved>> for Node<T, Unresolved> {
    fn resolve(self) -> Result<Node<Res, Resolved>, ()> {
        if let Ok(inner) = self.inner {
            return Ok(Node {
                inner: inner.resolve()?,
                span: self.span,
            });
        }
        Err(())
    }
}

impl<T: Resolvable<Res>, Res> Resolvable<Box<Res>> for Box<T> {
    fn resolve(self) -> Result<Box<Res>, ()> {
        Ok(Box::new((*self).resolve()?))
    }
}
