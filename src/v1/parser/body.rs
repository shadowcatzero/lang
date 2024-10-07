use std::fmt::{Debug, Write};

use super::{
    token::{Keyword, Symbol, Token},
    Node, Parsable, ParserErrors,
};
use crate::util::Padder;

use super::{Expr, ParserError, TokenCursor};

#[derive(Clone)]
pub struct Body {
    statements: Vec<Node<Statement>>,
}

#[derive(Clone)]
pub enum Statement {
    Let(String, Node<Expr>),
    Return(Node<Expr>),
    Expr(Node<Expr>),
}

impl Statement {
    pub fn ended_with_error(&self) -> bool {
        let expr = match self {
            Statement::Let(_, e) => e,
            Statement::Return(e) => e,
            Statement::Expr(e) => e,
        };
        expr.is_err() || expr.as_ref().is_ok_and(|e| e.ended_with_error())
    }
}

impl Parsable for Body {
    fn parse(cursor: &mut TokenCursor, errors: &mut ParserErrors) -> Result<Self, ParserError> {
        let mut statements = Vec::new();
        let statement_end = &[Symbol::Semicolon, Symbol::CloseCurly];
        cursor.expect_sym(Symbol::OpenCurly)?;
        if cursor.expect_peek()?.is_symbol(Symbol::CloseCurly) {
            cursor.next();
            return Ok(Self { statements });
        }
        let mut expect_semi = false;
        loop {
            let next = cursor.expect_peek()?;
            if next.is_symbol(Symbol::CloseCurly) {
                cursor.next();
                return Ok(Self { statements });
            }
            if next.is_symbol(Symbol::Semicolon) {
                cursor.next();
                expect_semi = false;
                continue;
            } else if expect_semi {
                errors.add(ParserError {
                    msg: "expected ';'".to_string(),
                    spans: vec![cursor.next_pos().char_span()],
                });
            }
            let statement: Node<Statement> = Node::parse(cursor, errors);
            expect_semi = true;
            if statement.is_err() || statement.as_ref().is_ok_and(|s| s.ended_with_error()) {
                let res = cursor
                    .seek(|t| t.is_symbol_and(|s| statement_end.contains(&s)))
                    .ok_or(ParserError::unexpected_end())?;
            }
            statements.push(statement);
        }
    }
}

impl Parsable for Statement {
    fn parse(cursor: &mut TokenCursor, errors: &mut ParserErrors) -> Result<Self, ParserError> {
        let next = cursor.expect_peek()?;
        Ok(match next.token {
            Token::Keyword(Keyword::Let) => {
                cursor.next();
                let name = cursor.expect_ident()?;
                cursor.expect_sym(Symbol::Equals)?;
                let expr = Node::parse(cursor, errors);
                Self::Let(name, expr)
            }
            Token::Keyword(Keyword::Return) => {
                cursor.next();
                Self::Return(Node::parse(cursor, errors))
            }
            _ => Self::Expr(Node::parse(cursor, errors)),
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
