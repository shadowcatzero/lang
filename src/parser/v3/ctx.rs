use std::ops::{Deref, DerefMut};

use crate::common::FileID;

use super::{
    CompilerMsg, CompilerOutput, Node, NodeParseResult, Parsable, ParsableWith, TokenCursor,
};

pub struct ParserCtx<'a> {
    pub cursor: TokenCursor<'a>,
    pub output: &'a mut CompilerOutput,
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
    pub fn err(&mut self, msg: CompilerMsg) {
        self.output.err(msg);
    }
    pub fn hint(&mut self, msg: CompilerMsg) {
        self.output.hint(msg);
    }
    pub fn parse<T: Parsable>(&mut self) -> NodeParseResult<T> {
        Node::parse(self)
    }
    pub fn maybe_parse<T>(&mut self) -> Option<NodeParseResult<T>>
    where
        Option<T>: Parsable,
    {
        Node::maybe_parse(self)
    }
    pub fn parse_with<T: ParsableWith>(&mut self, data: T::Data) -> NodeParseResult<T> {
        Node::parse_with(self, data)
    }
    pub fn new(file: FileID, string: &'a str, output: &'a mut CompilerOutput) -> Self {
        Self {
            cursor: TokenCursor::from_file_str(file, string),
            output,
        }
    }
}
