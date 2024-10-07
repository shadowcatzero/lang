use std::fmt::{Debug, Write};

use super::{
    token::{Keyword, Symbol, Token}, Node, NodeContainer, Parsable
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

impl Parsable for Body {
    fn parse(cursor: &mut TokenCursor) -> Result<Self, ParserError> {
        let mut statements = Vec::new();
        let statement_end = &[Symbol::Semicolon, Symbol::CloseCurly];
        cursor.expect_sym(Symbol::OpenCurly)?;
        if cursor.expect_peek()?.is_symbol(Symbol::CloseCurly) {
            cursor.next();
            return Ok(Self { statements });
        }
        loop {
            let next = cursor.expect_peek()?;
            if next.is_symbol(Symbol::CloseCurly) {
                cursor.next();
                return Ok(Self { statements });
            }
            statements.push(Node::parse(cursor));
            let next = cursor.expect_next()?;
            match next.token {
                Token::Symbol(Symbol::Semicolon) => continue,
                Token::Symbol(Symbol::CloseCurly) => return Ok(Self { statements }),
                _ => {
                    let start = next.span.start;
                    cursor
                        .seek(|t| t.is_symbol_and(|s| statement_end.contains(&s)))
                        .ok_or(ParserError::unexpected_end())?;
                    let end = cursor.prev_end();
                    let next = cursor.expect_next()?;
                    let span = start.to(end);
                    statements.push(Node::err(ParserError {
                        msg: "Unexpected tokens".to_string(),
                        spans: vec![span],
                    }, span));
                    if next.is_symbol(Symbol::CloseCurly) {
                        return Ok(Self { statements });
                    }
                }
            }
        }
    }
}

impl Parsable for Statement {
    fn parse(cursor: &mut TokenCursor) -> Result<Self, ParserError> {
        let next = cursor.expect_peek()?;
        Ok(match next.token {
            Token::Keyword(Keyword::Let) => {
                cursor.next();
                let name = cursor.expect_ident()?;
                cursor.expect_sym(Symbol::Equals)?;
                let expr = Node::parse(cursor);
                Self::Let(name, expr)
            }
            Token::Keyword(Keyword::Return) => {
                cursor.next();
                Self::Return(Node::parse(cursor))
            }
            _ => Self::Expr(Node::parse(cursor)),
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

impl NodeContainer for Body {
    fn children(&self) -> Vec<Node<Box<dyn NodeContainer>>> {
        self.statements.iter().map(|f| f.containerr()).collect()
    }
}

impl NodeContainer for Statement {
    fn children(&self) -> Vec<Node<Box<dyn NodeContainer>>> {
        match self {
            Statement::Let(_, e) => vec![e.containerr()],
            Statement::Return(e) => vec![e.containerr()],
            Statement::Expr(e) => vec![e.containerr()],
        }
    }
}
