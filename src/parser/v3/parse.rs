use std::{
    convert::Infallible,
    ops::{ControlFlow, FromResidual, Try},
};

use super::{Node, ParserError, ParserErrors, TokenCursor};

pub enum ParseResult<T> {
    Ok(T),
    Recover(T),
    Err(ParserError),
    SubErr,
}

impl<T> ParseResult<T> {
    pub fn from_recover(data: T, recover: bool) -> Self {
        if recover {
            Self::Recover(data)
        } else {
            Self::Ok(data)
        }
    }
}

impl<T> Try for ParseResult<T> {
    type Output = Result<T, T>;
    type Residual = Option<ParserError>;
    fn from_output(output: Self::Output) -> Self {
        match output {
            Ok(v) => Self::Ok(v),
            Err(v) => Self::Recover(v),
        }
    }
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            ParseResult::Ok(v) => ControlFlow::Continue(Ok(v)),
            ParseResult::Recover(v) => ControlFlow::Continue(Err(v)),
            ParseResult::Err(e) => ControlFlow::Break(Some(e)),
            ParseResult::SubErr => ControlFlow::Break(None),
        }
    }
}

impl<T> FromResidual for ParseResult<T> {
    fn from_residual(residual: <Self as Try>::Residual) -> Self {
        match residual {
            Some(err) => Self::Err(err),
            None => Self::SubErr,
        }
    }
}

impl<T> FromResidual<Result<Infallible, ParserError>> for ParseResult<T> {
    fn from_residual(residual: Result<Infallible, ParserError>) -> Self {
        match residual {
            Err(e) => Self::Err(e),
        }
    }
}

impl<T, U> FromResidual<ParseResult<T>> for ParseResult<U> {
    fn from_residual(residual: ParseResult<T>) -> Self {
        match residual {
            ParseResult::Err(e) => Self::Err(e),
            ParseResult::SubErr => Self::SubErr,
            _ => unreachable!(),
        }
    }
}

pub struct NodeParseResult<T> {
    pub node: Node<T>,
    pub recover: bool,
}

impl<T> NodeParseResult<T> {
    pub fn map<F: FnOnce(Node<T>) -> U, U>(self, op: F) -> ParseResult<U> {
        let res = op(self.node);
        if self.recover {
            ParseResult::Recover(res)
        } else {
            ParseResult::Ok(res)
        }
    }
}

impl<T> Try for NodeParseResult<T> {
    type Output = Node<T>;
    type Residual = ParseResult<T>;

    fn from_output(output: Self::Output) -> Self {
        Self {
            node: output,
            recover: false,
        }
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        if self.recover {
            ControlFlow::Break(ParseResult::SubErr)
        } else {
            ControlFlow::Continue(self.node)
        }
    }
}

impl<T> FromResidual for NodeParseResult<T> {
    fn from_residual(_: <Self as Try>::Residual) -> Self {
        // I hope this is unreachable ???
        unreachable!()
    }
}

pub trait Parsable: Sized {
    fn parse(cursor: &mut TokenCursor, errors: &mut ParserErrors) -> ParseResult<Self>;
}

pub trait MaybeParsable: Sized {
    fn maybe_parse(
        cursor: &mut TokenCursor,
        errors: &mut ParserErrors,
    ) -> Result<Option<Self>, ParserError>;
}

impl<T: Parsable> Node<T> {
    pub fn parse(cursor: &mut TokenCursor, errors: &mut ParserErrors) -> NodeParseResult<T> {
        let start = cursor.next_pos();
        let (inner, recover) = match T::parse(cursor, errors) {
            ParseResult::Ok(v) => (Some(v), false),
            ParseResult::Recover(v) => (Some(v), true),
            ParseResult::Err(e) => {
                errors.add(e);
                (None, true)
            }
            ParseResult::SubErr => (None, true),
        };
        let end = cursor.prev_end();
        NodeParseResult {
            node: Self {
                inner,
                span: start.to(end),
            },
            recover,
        }
    }
}

impl<T: MaybeParsable> Node<T> {
    pub fn maybe_parse(cursor: &mut TokenCursor, errors: &mut ParserErrors) -> Option<Self> {
        let start = cursor.next_pos();
        let inner = match T::maybe_parse(cursor, errors) {
            Ok(v) => Some(v?),
            Err(e) => {
                errors.add(e);
                None
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
    fn parse_node(cursor: &mut TokenCursor, errors: &mut ParserErrors) -> NodeParseResult<Self>
    where
        Self: Sized;
}
impl<T: Parsable> NodeParsable for T {
    fn parse_node(cursor: &mut TokenCursor, errors: &mut ParserErrors) -> NodeParseResult<Self>
    where
        Self: Sized,
    {
        Node::<Self>::parse(cursor, errors)
    }
}
