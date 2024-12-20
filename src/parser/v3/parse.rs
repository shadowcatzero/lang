use std::{
    convert::Infallible,
    ops::{ControlFlow, FromResidual, Try},
};

use crate::ir::FilePos;

use super::{Node, ParserCtx, ParserMsg};

pub enum ParseResult<T> {
    Ok(T),
    Recover(T),
    Err(ParserMsg),
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
    type Output = T;
    type Residual = Option<ParserMsg>;
    fn from_output(output: Self::Output) -> Self {
        Self::Ok(output)
    }
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            ParseResult::Ok(v) => ControlFlow::Continue(v),
            // TODO: this is messed up; need to break w a Result<Option<T>> or smth :woozy:
            ParseResult::Recover(v) => ControlFlow::Break(None),
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

impl<T> FromResidual<Result<Infallible, ParserMsg>> for ParseResult<T> {
    fn from_residual(residual: Result<Infallible, ParserMsg>) -> Self {
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
    fn parse(ctx: &mut ParserCtx) -> ParseResult<Self>;
}

pub trait MaybeParsable: Sized {
    fn maybe_parse(ctx: &mut ParserCtx) -> Result<Option<Self>, ParserMsg>;
}

impl<T: Parsable> Node<T> {
    pub fn parse(ctx: &mut ParserCtx) -> NodeParseResult<T> {
        let start = ctx.peek().map(|t| t.span.start).unwrap_or(FilePos::start());
        let (inner, recover) = match T::parse(ctx) {
            ParseResult::Ok(v) => (Some(v), false),
            ParseResult::Recover(v) => (Some(v), true),
            ParseResult::Err(e) => {
                ctx.err(e);
                (None, true)
            }
            ParseResult::SubErr => (None, true),
        };
        let end = ctx.prev_end();
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
    pub fn maybe_parse(ctx: &mut ParserCtx) -> Option<Self> {
        let start = ctx.next_start();
        let inner = match T::maybe_parse(ctx) {
            Ok(v) => Some(v?),
            Err(e) => {
                ctx.err(e);
                None
            }
        };
        let end = ctx.prev_end();
        Some(Self {
            inner,
            span: start.to(end),
        })
    }
}

pub trait NodeParsable {
    fn parse_node(ctx: &mut ParserCtx) -> NodeParseResult<Self>
    where
        Self: Sized;
}
impl<T: Parsable> NodeParsable for T {
    fn parse_node(ctx: &mut ParserCtx) -> NodeParseResult<Self>
    where
        Self: Sized,
    {
        Node::<Self>::parse(ctx)
    }
}
