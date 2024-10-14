use std::fmt::{Debug, Write};
use super::{
    Expr, Keyword, MaybeResolved, Node, Parsable, ParseResult, ParserErrors, Resolvable, Resolved, Symbol, Token, TokenCursor, Unresolved
};

pub enum Statement<R: MaybeResolved> {
    Let(String, Node<Expr<R>, R>),
    Return(Node<Expr<R>, R>),
    Expr(Node<Expr<R>, R>),
}

impl Parsable for Statement<Unresolved> {
    fn parse(cursor: &mut TokenCursor, errors: &mut ParserErrors) -> ParseResult<Self> {
        let next = cursor.expect_peek()?;
        match next.token {
            Token::Keyword(Keyword::Let) => {
                cursor.next();
                let name = cursor.expect_ident()?;
                cursor.expect_sym(Symbol::Equals)?;
                Node::parse(cursor, errors).map(|expr| Self::Let(name, expr))
            }
            Token::Keyword(Keyword::Return) => {
                cursor.next();
                Node::parse(cursor, errors).map(Self::Return)
            }
            _ => Node::parse(cursor, errors).map(Self::Expr),
        }
    }
}

impl Resolvable<Statement<Resolved>> for Statement<Unresolved> {
    fn resolve(self) -> Result<Statement<Resolved>, ()> {
        Ok(match self {
            Self::Let(i, e) => Statement::Let(i, e.resolve()?),
            Self::Return(e) => Statement::Return(e.resolve()?),
            Self::Expr(e) => Statement::Expr(e.resolve()?),
        })
    }
}

impl Debug for Statement<Unresolved> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Let(n, e) => {
                f.write_str("let ")?;
                f.write_str(n)?;
                f.write_str(" = ")?;
                e.fmt(f)?;
                f.write_char(';')?;
            }
            Statement::Return(e) => {
                f.write_str("return ")?;
                e.fmt(f)?;
                f.write_char(';')?;
            }
            Statement::Expr(e) => {
                e.fmt(f)?;
                f.write_char(';')?;
            }
        }
        Ok(())
    }
}
