use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use super::{FileSpan, ParserError, ParserErrors, TokenCursor};

#[derive(Clone)]
pub struct Node<T> {
    pub inner: Result<T, ()>,
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

impl<T: Parsable> Node<T> {
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

impl<T: MaybeParsable> Node<T> {
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

impl<T> Node<T> {
    pub fn new(inner: T, span: FileSpan) -> Self {
        Self {
            inner: Ok(inner),
            span,
        }
    }
    pub fn bx(self) -> Node<Box<T>> {
        Node {
            inner: self.inner.map(|v| Box::new(v)),
            span: self.span,
        }
    }
}

impl<T> Node<Box<T>> {
    pub fn unbx(self) -> Node<T> {
        Node {
            inner: self.inner.map(|v| *v),
            span: self.span,
        }
    }
}
impl<T> Deref for Node<T> {
    type Target = Result<T, ()>;
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
            Ok(v) => v.fmt(f),
            Err(_) => f.write_str("{error}"),
        }
    }
}
