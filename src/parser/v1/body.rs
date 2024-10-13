use std::fmt::{Debug, Write};

use super::{
    token::Symbol, MaybeResolved, Node, NodeParsable, Parsable, ParserError, ParserErrors,
    Resolvable, Resolved, Statement, TokenCursor, Unresolved,
};
use crate::util::Padder;

pub struct Body<R: MaybeResolved> {
    statements: Vec<Node<Statement<R>, R>>,
}

impl Parsable for Body<Unresolved> {
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
            let statement = Statement::parse_node(cursor, errors);
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

impl Resolvable<Body<Resolved>> for Body<Unresolved> {
    fn resolve(self) -> Result<Body<Resolved>, ()> {
        Ok(Body {
            statements: self
                .statements
                .into_iter()
                .map(|s| s.resolve())
                .collect::<Result<_, _>>()?,
        })
    }
}

impl Debug for Body<Unresolved> {
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
