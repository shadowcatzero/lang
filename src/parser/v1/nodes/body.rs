use std::fmt::{Debug, Write};

use super::{
    token::Symbol, MaybeResolved, Node, NodeParsable, Parsable, ParseResult, ParserError,
    ParserErrors, Resolvable, Resolved, Statement, TokenCursor, Unresolved,
};
use crate::util::Padder;

pub struct Body<R: MaybeResolved> {
    statements: Vec<Node<Statement<R>, R>>,
}

impl Parsable for Body<Unresolved> {
    fn parse(cursor: &mut TokenCursor, errors: &mut ParserErrors) -> ParseResult<Self> {
        let mut statements = Vec::new();
        let statement_end = &[Symbol::Semicolon, Symbol::CloseCurly];
        cursor.expect_sym(Symbol::OpenCurly)?;
        if cursor.expect_peek()?.is_symbol(Symbol::CloseCurly) {
            cursor.next();
            return ParseResult::Ok(Self { statements });
        }
        let mut expect_semi = false;
        let mut recover = false;
        loop {
            let Some(next) = cursor.peek() else {
                recover = true;
                errors.add(ParserError::unexpected_end());
                break;
            };
            if next.is_symbol(Symbol::CloseCurly) {
                cursor.next();
                break;
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
            let res = Statement::parse_node(cursor, errors);
            statements.push(res.node);
            expect_semi = true;
            if res.recover
                && cursor
                    .seek(|t| t.is_symbol_and(|s| statement_end.contains(&s)))
                    .is_none()
            {
                recover = true;
                break;
            }
        }
        ParseResult::from_recover(Self { statements }, recover)
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
