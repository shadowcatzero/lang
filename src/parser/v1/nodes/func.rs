use super::{
    Body, Ident, Keyword, MaybeResolved, Node, Parsable, ParseResult, ParserErrors, Resolvable,
    Resolved, Symbol, TokenCursor, Unresolved,
};
use std::fmt::Debug;

pub struct Function<R: MaybeResolved> {
    pub name: Node<Ident, R>,
    pub body: Node<Body<R>, R>,
}

impl Parsable for Function<Unresolved> {
    fn parse(cursor: &mut TokenCursor, errors: &mut ParserErrors) -> ParseResult<Self> {
        cursor.expect_kw(Keyword::Fn)?;
        let name = Node::parse(cursor, errors)?;
        cursor.expect_sym(Symbol::OpenParen)?;
        cursor.expect_sym(Symbol::CloseParen)?;
        Node::parse(cursor, errors).map(|body| Self { name, body })
    }
}

impl Debug for Function<Unresolved> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("fn ")?;
        self.name.fmt(f)?;
        f.write_str("() ")?;
        self.body.fmt(f)?;
        Ok(())
    }
}

impl Resolvable<Function<Resolved>> for Function<Unresolved> {
    fn resolve(self) -> Result<Function<Resolved>, ()> {
        Ok(Function {
            name: self.name.resolve()?,
            body: self.body.resolve()?,
        })
    }
}
