use super::{Body, Ident, Keyword, Node, Parsable, ParseResult, ParserErrors, Symbol, TokenCursor};
use std::fmt::Debug;

pub struct Function {
    pub name: Node<Ident>,
    pub body: Node<Body>,
}

impl Parsable for Function {
    fn parse(cursor: &mut TokenCursor, errors: &mut ParserErrors) -> ParseResult<Self> {
        cursor.expect_kw(Keyword::Fn)?;
        let name = Node::parse(cursor, errors)?;
        cursor.expect_sym(Symbol::OpenParen)?;
        cursor.expect_sym(Symbol::CloseParen)?;
        Node::parse(cursor, errors).map(|body| Self { name, body })
    }
}

impl Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("fn ")?;
        self.name.fmt(f)?;
        f.write_str("() ")?;
        self.body.fmt(f)?;
        Ok(())
    }
}

