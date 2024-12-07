use std::fmt::{Debug, Write};

use super::{
    token::Symbol, Node, NodeParsable, Parsable, ParseResult, ParserCtx, ParserMsg, PStatement,
};
use crate::util::Padder;

pub struct PBlock {
    pub statements: Vec<Node<PStatement>>,
    pub result: Option<Node<Box<PStatement>>>,
}

impl Parsable for PBlock {
    fn parse(ctx: &mut ParserCtx) -> ParseResult<Self> {
        let mut statements = Vec::new();
        let mut result = None;
        ctx.expect_sym(Symbol::OpenCurly)?;
        if ctx.expect_peek()?.is_symbol(Symbol::CloseCurly) {
            ctx.next();
            return ParseResult::Ok(Self { statements, result });
        }
        let mut expect_semi = false;
        let mut recover = false;
        loop {
            let Some(next) = ctx.peek() else {
                recover = true;
                ctx.err(ParserMsg::unexpected_end());
                break;
            };
            if next.is_symbol(Symbol::CloseCurly) {
                ctx.next();
                break;
            }
            if next.is_symbol(Symbol::Semicolon) {
                ctx.next();
                expect_semi = false;
                continue;
            } else if expect_semi {
                ctx.err(ParserMsg {
                    msg: "expected ';'".to_string(),
                    spans: vec![ctx.next_start().char_span()],
                });
            }
            let res = PStatement::parse_node(ctx);
            statements.push(res.node);
            expect_semi = true;
            if res.recover {
                ctx.seek_syms(&[Symbol::Semicolon, Symbol::CloseCurly]);
                if ctx.peek().is_none() {
                    recover = true;
                    break;
                }
            }
        }
        if expect_semi {
            if let Some(s) = statements.pop() {
                result = Some(s.bx());
            }
        }
        ParseResult::from_recover(Self { statements, result }, recover)
    }
}

impl Debug for PBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.statements.is_empty() || self.result.is_some() {
            f.write_str("{\n    ")?;
            let mut padder = Padder::new(f);
            for s in &self.statements {
                // they don't expose wrap_buf :grief:
                padder.write_str(&format!("{s:?};\n"))?;
            }
            if let Some(res) = &self.result {
                padder.write_str(&format!("{res:?}\n"))?;
            }
            f.write_char('}')?;
        } else {
            f.write_str("{}")?;
        }
        Ok(())
    }
}
