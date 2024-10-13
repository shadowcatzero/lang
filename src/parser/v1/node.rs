use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use super::{FileSpan, ParserError, ParserErrors, TokenCursor};

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

pub trait Parsable: Sized {
    fn parse(cursor: &mut TokenCursor, errors: &mut ParserErrors) -> Result<Self, ParserError>;
}

pub trait MaybeParsable: Sized {
    fn maybe_parse(
        cursor: &mut TokenCursor,
        errors: &mut ParserErrors,
    ) -> Result<Option<Self>, ParserError>;
}

impl<T: Parsable> Node<T, Unresolved> {
    pub fn parse(cursor: &mut TokenCursor, errors: &mut ParserErrors) -> Self {
        let start = cursor.next_pos();
        let inner = T::parse(cursor, errors).map_err(|e| errors.add(e));
        let end = cursor.prev_end();
        Self {
            inner,
            span: start.to(end),
        }
    }
}

impl<T: MaybeParsable> Node<T, Unresolved> {
    pub fn maybe_parse(cursor: &mut TokenCursor, errors: &mut ParserErrors) -> Option<Self> {
        let start = cursor.next_pos();
        let inner = match T::maybe_parse(cursor, errors) {
            Ok(v) => Ok(v?),
            Err(e) => {
                errors.add(e);
                Err(())
            }
        };
        let end = cursor.prev_end();
        Some(Self {
            inner,
            span: start.to(end),
        })
    }
}

pub trait NodeParsable {
    fn parse_node(cursor: &mut TokenCursor, errors: &mut ParserErrors) -> Node<Self, Unresolved>
    where
        Self: Sized;
}
impl<T: Parsable> NodeParsable for T {
    fn parse_node(cursor: &mut TokenCursor, errors: &mut ParserErrors) -> Node<Self, Unresolved>
    where
        Self: Sized,
    {
        Node::<Self, Unresolved>::parse(cursor, errors)
    }
}

impl<T> Node<T, Unresolved> {
    pub fn new_unres(inner: T, span: FileSpan) -> Self {
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
