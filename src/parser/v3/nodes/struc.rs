use std::fmt::Debug;

use super::{
    util::parse_list, Keyword, Node, PExpr, PFieldDef, PIdent, PType, PVarDef, Parsable,
    ParseResult, ParserCtx, CompilerMsg, Symbol,
};

#[derive(Debug)]
pub struct PStruct {
    pub name: Node<PIdent>,
    pub fields: PStructFields,
}

#[derive(Debug)]
pub struct PConstruct {
    pub name: Node<PIdent>,
    pub fields: PConstructFields,
}

#[derive(Debug)]
pub enum PStructFields {
    Named(Vec<Node<PVarDef>>),
    Tuple(Vec<Node<PType>>),
    None,
}

#[derive(Debug)]
pub enum PConstructFields {
    Named(Vec<Node<PFieldDef>>),
    Tuple(Vec<Node<PExpr>>),
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
            let msg = CompilerMsg::unexpected_token(next, "`;`, `(`, or `{`");
            ctx.err(msg);
            return ParseResult::Recover(PStruct {
                name,
                fields: PStructFields::None,
            });
        };
        ParseResult::Ok(PStruct { name, fields })
    }
}

impl Parsable for PConstruct {
    fn parse(ctx: &mut ParserCtx) -> ParseResult<Self> {
        ctx.expect_kw(Keyword::Struct)?;
        let name = ctx.parse()?;
        let next = ctx.expect_peek()?;
        let fields = if next.is_symbol(Symbol::Semicolon) {
            ctx.next();
            PConstructFields::None
        } else if next.is_symbol(Symbol::OpenCurly) {
            ctx.next();
            PConstructFields::Named(parse_list(ctx, Symbol::CloseCurly)?)
        } else if next.is_symbol(Symbol::OpenParen) {
            ctx.next();
            PConstructFields::Tuple(parse_list(ctx, Symbol::CloseParen)?)
        } else {
            let msg = CompilerMsg::unexpected_token(next, "`;`, `(`, or `{`");
            ctx.err(msg);
            return ParseResult::Recover(PConstruct {
                name,
                fields: PConstructFields::None,
            });
        };
        ParseResult::Ok(PConstruct { name, fields })
    }
}
