use std::fmt::{Debug, Write};

use super::{
    token::Symbol, CompilerMsg, Node, NodeParsable, PStatementLike, ParseResult, ParserCtx,
};
use crate::{
    parser::{ParsableWith, TokenInstance},
    util::Padder,
};

pub struct PBlock {
    pub statements: Vec<Node<PStatementLike>>,
    pub ret_last: bool,
}

impl ParsableWith for PBlock {
    type Data = Option<Symbol>;
    fn parse(ctx: &mut ParserCtx, end: Option<Symbol>) -> ParseResult<Self> {
        let mut statements = Vec::new();
        let is_end = |t: &TokenInstance| -> bool { end.map(|e| t.is_symbol(e)).unwrap_or(false) };
        if ctx.peek().is_none_or(is_end) {
            ctx.next();
            return ParseResult::Ok(Self {
                statements,
                ret_last: false,
            });
        }
        let mut expect_semi = false;
        let mut recover = false;
        loop {
            let Some(next) = ctx.peek() else {
                if end.is_some() {
                    recover = true;
                    ctx.err(CompilerMsg::unexpected_end());
                }
                break;
            };
            if is_end(next) {
                ctx.next();
                break;
            }
            if next.is_symbol(Symbol::Semicolon) {
                ctx.next();
                expect_semi = false;
                continue;
            } else if expect_semi {
                ctx.err(CompilerMsg {
                    msg: "expected ';'".to_string(),
                    spans: vec![ctx.next_start().char_span()],
                });
            }
            let res = PStatementLike::parse_node(ctx);
            expect_semi = res
                .node
                .as_ref()
                .is_some_and(|s| matches!(s, PStatementLike::Statement(..)));
            statements.push(res.node);
            if res.recover {
                ctx.seek_syms(&[Symbol::Semicolon, Symbol::CloseCurly]);
                if ctx.peek().is_none() {
                    recover = true;
                    break;
                }
            }
        }
        ParseResult::from_recover(
            Self {
                statements,
                ret_last: expect_semi,
            },
            recover,
        )
    }
}

impl Debug for PBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.statements.is_empty() {
            f.write_str("{\n    ")?;
            let mut padder = Padder::new(f);
            let mut end = self.statements.len();
            if self.ret_last {
                end -= 1;
            }
            for i in 0..end {
                let s = &self.statements[i];
                // they don't expose wrap_buf :grief:
                padder.write_str(&format!("{s:?};\n"))?;
            }
            if self.ret_last
                && let Some(s) = self.statements.last()
            {
                padder.write_str(&format!("{s:?}\n"))?;
            }
            f.write_char('}')?;
        } else {
            f.write_str("{}")?;
        }
        Ok(())
    }
}
