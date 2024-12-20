use std::ops::{Deref, DerefMut};

use super::{MaybeParsable, Node, NodeParseResult, Parsable, ParserMsg, ParserOutput, TokenCursor};

pub struct ParserCtx<'a> {
    pub cursor: TokenCursor<'a>,
    pub output: ParserOutput,
}

impl<'a> Deref for ParserCtx<'a> {
    type Target = TokenCursor<'a>;

    fn deref(&self) -> &Self::Target {
        &self.cursor
    }
}

impl DerefMut for ParserCtx<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cursor
    }
}

impl<'a> ParserCtx<'a> {
    pub fn err(&mut self, msg: ParserMsg) {
        self.output.err(msg);
    }
    pub fn hint(&mut self, msg: ParserMsg) {
        self.output.hint(msg);
    }
    pub fn parse<T: Parsable>(&mut self) -> NodeParseResult<T> {
        Node::parse(self)
    }
    pub fn maybe_parse<T: MaybeParsable>(&mut self) -> Option<Node<T>> {
        Node::maybe_parse(self)
    }
}

impl<'a> From<TokenCursor<'a>> for ParserCtx<'a> {
    fn from(cursor: TokenCursor<'a>) -> Self {
        Self {
            cursor,
            output: ParserOutput::new(),
        }
    }
}

impl<'a> From<&'a str> for ParserCtx<'a> {
    fn from(string: &'a str) -> Self {
        Self {
            cursor: TokenCursor::from(string),
            output: ParserOutput::new(),
        }
    }
}
