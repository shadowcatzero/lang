use super::{
    Expr, Ident, Keyword, Node, Parsable, ParseResult, ParserErrors, Symbol, Token, TokenCursor,
};
use std::fmt::{Debug, Write};

pub enum Statement {
    Let(Node<Ident>, Node<Expr>),
    Return(Node<Expr>),
    Expr(Node<Expr>),
}

impl Parsable for Statement {
    fn parse(cursor: &mut TokenCursor, errors: &mut ParserErrors) -> ParseResult<Self> {
        let next = cursor.expect_peek()?;
        match next.token {
            Token::Keyword(Keyword::Let) => {
                cursor.next();
                let name = Node::parse(cursor, errors)?;
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

impl Debug for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Let(n, e) => {
                f.write_str("let ")?;
                n.fmt(f);
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
