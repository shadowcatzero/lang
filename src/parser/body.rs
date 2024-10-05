use std::fmt::{Debug, Write};

use crate::token::{Keyword, Symbol, Token};
use crate::util::Padder;

use super::cursor::TokenCursor;
use super::error::{unexpected_token, ParserError};
use super::Expr;

pub struct Body {
    statements: Vec<Statement>,
}

pub enum Statement {
    Let(String, Expr),
    Return(Expr),
    Expr(Expr),
}

impl Body {
    pub fn parse(cursor: &mut TokenCursor) -> Result<Self, ParserError> {
        let mut statements = Vec::new();
        cursor.expect_sym(Symbol::OpenCurly)?;
        loop {
            let next = cursor.expect_peek()?;
            if next.is_symbol(Symbol::CloseCurly) {
                cursor.next();
                return Ok(Self { statements });
            }
            statements.push(Statement::parse(cursor)?);
        }
    }
}

impl Statement {
    pub fn parse(cursor: &mut TokenCursor) -> Result<Self, ParserError> {
        let next = cursor.expect_peek()?;
        Ok(match next.token {
            Token::Keyword(Keyword::Let) => {
                cursor.next();
                let name = cursor.expect_ident()?;
                cursor.expect_sym(Symbol::Equals)?;
                let expr = Expr::parse(cursor)?;
                cursor.expect_sym(Symbol::Semicolon)?;
                Self::Let(name, expr)
            }
            Token::Keyword(Keyword::Return) => {
                cursor.next();
                let expr = Expr::parse(cursor)?;
                cursor.expect_sym(Symbol::Semicolon)?;
                Self::Return(expr)
            }
            _ => {
                let expr = Expr::parse(cursor)?;
                let next = cursor.expect_peek()?;
                if next.is_symbol(Symbol::Semicolon) {
                    cursor.next();
                    Self::Expr(expr)
                } else if next.is_symbol(Symbol::CloseCurly) {
                    Self::Return(expr)
                } else {
                    return unexpected_token(next, "a ';' or '}'");
                }
            }
        })
    }
}

impl Debug for Statement {
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

impl Debug for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.statements.first().is_some() {
            f.write_str("{\n    ")?;
            let mut padder = Padder::new(f);
            for s in &self.statements {
                // they don't expose wrap_buf :grief:
                padder.write_str(&format!("{s:?}\n"))?;
            }
            f.write_char('}')?;
        } else {
            f.write_str("{}")?;
        }
        Ok(())
    }
}
