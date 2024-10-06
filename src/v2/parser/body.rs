use std::collections::HashSet;
use std::fmt::{Debug, Write};
use std::sync::LazyLock;

use crate::util::Padder;

use super::util::WHITESPACE_SET;
use super::CharCursor;
use super::Expr;
use super::ParserError;

static NAME_END: LazyLock<HashSet<char>> = LazyLock::new(|| {
    let mut set = WHITESPACE_SET.clone();
    set.extend(&['(']);
    set
});

pub struct Body {
    statements: Vec<Statement>,
}

pub enum Statement {
    Let(String, Expr),
    Return(Expr),
    Expr(Expr),
}

impl Body {
    pub fn parse(cursor: &mut CharCursor) -> Result<Self, ParserError> {
        cursor.skip_whitespace();
        let mut statements = Vec::new();
        cursor.expect_char('{')?;
        loop {
            cursor.skip_whitespace();
            let next = cursor.expect_peek()?;
            if next == '}' {
                cursor.next();
                return Ok(Self { statements });
            }
            statements.push(Statement::parse(cursor)?);
        }
    }
}

impl Statement {
    pub fn parse(cursor: &mut CharCursor) -> Result<Self, ParserError> {
        cursor.skip_whitespace();
        Ok(if cursor.advance_if_str("let", &WHITESPACE_SET) {
            cursor.skip_whitespace();
            let name = cursor.until(&NAME_END);
            if name.is_empty() {
                return Err(ParserError::at(
                    cursor.pos(),
                    "Expected variable name".to_string(),
                ));
            }
            cursor.skip_whitespace();
            cursor.expect_char('=')?;
            let expr = Expr::parse(cursor)?;
            cursor.skip_whitespace();
            cursor.expect_char(';')?;
            Self::Let(name, expr)
        } else if cursor.advance_if_str("return", &WHITESPACE_SET) {
            let expr = Expr::parse(cursor)?;
            cursor.skip_whitespace();
            cursor.expect_char(';')?;
            Self::Return(expr)
        } else {
            let expr = Expr::parse(cursor)?;
            match cursor.expect_peek()? {
                ';' => {
                    cursor.next();
                    Self::Expr(expr)
                }
                '}' => Self::Return(expr),
                _ => {
                    cursor.next();
                    return Err(ParserError::at(
                        cursor.prev_pos(),
                        "unexpected end of statement; expected a ';' or '}'".to_string(),
                    ));
                }
            }
        })
    }
}

impl Debug for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Let(n, e) => {
                write!(f, "let {n} = {e:?};")?;
            }
            Statement::Return(e) => {
                write!(f, "return {e:?};")?;
            }
            Statement::Expr(e) => {
                write!(f, "{e:?};")?;
            }
        }
        Ok(())
    }
}

impl Debug for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.statements.first().is_some() {
            write!(f, "{{\n    ")?;
            let mut padder = Padder::new(f);
            for s in &self.statements {
                // they don't expose wrap_buf :grief:
                write!(padder, "{s:?}\n")?;
            }
            write!(f, "}}")?;
        } else {
            write!(f, "{{}}")?;
        }
        Ok(())
    }
}
