use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use super::{FileSpan, ParserError, TokenCursor};

#[derive(Clone)]
pub struct Node<T> {
    pub inner: Result<T, ParserError>,
    pub span: FileSpan,
}

pub trait Parsable: Sized {
    fn parse(cursor: &mut TokenCursor) -> Result<Self, ParserError>;
}

pub trait MaybeParsable: Sized {
    fn maybe_parse(cursor: &mut TokenCursor) -> Result<Option<Self>, ParserError>;
}

pub trait NodeContainer {
    fn children(&self) -> Vec<Node<Box<dyn NodeContainer>>>;
}

impl<T: NodeContainer + ?Sized> NodeContainer for Node<Box<T>> {
    fn children(&self) -> Vec<Node<Box<dyn NodeContainer>>> {
        match &self.inner {
            Ok(v) => v.children(),
            Err(_) => Vec::new(),
        }
    }
}

impl<T: Parsable> Node<T> {
    pub fn parse(cursor: &mut TokenCursor) -> Self {
        let start = cursor.next_pos();
        let inner = T::parse(cursor);
        let end = cursor.prev_end();
        Self {
            inner,
            span: start.to(end),
        }
    }
}

impl<T: MaybeParsable> Node<T> {
    pub fn maybe_parse(cursor: &mut TokenCursor) -> Option<Self> {
        let start = cursor.next_pos();
        let inner = match T::maybe_parse(cursor) {
            Ok(v) => Ok(v?),
            Err(e) => Err(e),
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
    pub fn err(inner: ParserError, span: FileSpan) -> Self {
        Self {
            inner: Err(inner),
            span,
        }
    }
    pub fn take(self) -> Result<T, ParserError> {
        self.inner
    }
    pub fn bx(self) -> Node<Box<T>> {
        Node {
            inner: self.inner.map(|v| Box::new(v)),
            span: self.span,
        }
    }
}

impl<T: NodeContainer + Clone + 'static> Node<T> {
    pub fn containerr(&self) -> Node<Box<dyn NodeContainer>> {
        Node {
            inner: self.clone().inner.map(|v| Box::new(v) as Box<dyn NodeContainer>),
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
impl<T: NodeContainer + Clone + 'static> Node<Box<T>> {
    pub fn container(&self) -> Node<Box<dyn NodeContainer>> {
        Node {
            inner: self.clone().inner.map(|v| v as Box<dyn NodeContainer>),
            span: self.span,
        }
    }
}

impl<T> Deref for Node<T> {
    type Target = Result<T, ParserError>;
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
