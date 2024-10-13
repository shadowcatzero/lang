use super::{
    Body, Keyword, MaybeResolved, Node, Parsable, ParserError, ParserErrors, Resolvable, Resolved, Symbol, TokenCursor, Unresolved
};
use std::fmt::Debug;

pub struct Function<R: MaybeResolved> {
    pub name: String,
    pub body: Node<Body<R>, R>,
}

impl Parsable for Function<Unresolved> {
    fn parse(cursor: &mut TokenCursor, errors: &mut ParserErrors) -> Result<Self, ParserError> {
        cursor.expect_kw(Keyword::Fn)?;
        let name = cursor.expect_ident()?;
        cursor.expect_sym(Symbol::OpenParen)?;
        cursor.expect_sym(Symbol::CloseParen)?;
        let body = Node::parse(cursor, errors);
        Ok(Self { name, body })
    }
}

impl Debug for Function<Unresolved> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("fn ")?;
        f.write_str(&self.name)?;
        f.write_str("() ")?;
        self.body.fmt(f)?;
        Ok(())
    }
}

impl Resolvable<Function<Resolved>> for Function<Unresolved> {
    fn resolve(self) -> Result<Function<Resolved>, ()> {
        Ok(Function {
            name: self.name,
            body: self.body.resolve()?
        })
    }
}
