use std::fmt::Debug;

use super::{
    util::parse_list, PIdent, Keyword, Node, Parsable, ParseResult, ParserCtx, ParserMsg, Symbol,
    PType, PVarDef,
};

#[derive(Debug)]
pub struct PStruct {
    pub name: Node<PIdent>,
    pub fields: PStructFields,
}

#[derive(Debug)]
pub enum PStructFields {
    Named(Vec<Node<PVarDef>>),
    Tuple(Vec<Node<PType>>),
    None,
}

impl Parsable for PStruct {
    fn parse(ctx: &mut ParserCtx) -> ParseResult<Self> {
        ctx.expect_kw(Keyword::Struct)?;
        let name = ctx.parse()?;
        let next = ctx.expect_peek()?;
        let fields = if next.is_symbol(Symbol::Semicolon) {
            ctx.next();
            PStructFields::None
        } else if next.is_symbol(Symbol::OpenCurly) {
            ctx.next();
            PStructFields::Named(parse_list(ctx, Symbol::CloseCurly)?)
        } else if next.is_symbol(Symbol::OpenParen) {
            ctx.next();
            PStructFields::Tuple(parse_list(ctx, Symbol::CloseParen)?)
        } else {
            let msg = ParserMsg::unexpected_token(next, "`;`, `(`, or `{`");
            ctx.err(msg);
            return ParseResult::Recover(PStruct {
                name,
                fields: PStructFields::None,
            });
        };
        ParseResult::Ok(PStruct { name, fields })
    }
}
