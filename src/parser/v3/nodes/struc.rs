use std::fmt::Debug;

use crate::parser::ParsableWith;

use super::{
    util::parse_list, CompilerMsg, Keyword, Node, PExpr, PFieldDef, PIdent, PType, PGenericDef,
    PVarDef, Parsable, ParseResult, ParserCtx, Symbol,
};

#[derive(Debug)]
pub struct PStruct {
    pub name: Node<PIdent>,
    pub generics: Vec<Node<PGenericDef>>,
    pub fields: PStructFields,
}

#[derive(Debug)]
pub struct PConstruct {
    pub name: Node<PType>,
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
        let name = ctx.parse()?;
        let mut next = ctx.expect_peek()?;
        let args = if next.is_symbol(Symbol::OpenAngle) {
            ctx.next();
            let res = parse_list(ctx, Symbol::CloseAngle)?;
            next = ctx.expect_peek()?;
            res
        } else {
            Vec::new()
        };
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
                generics: args,
                fields: PStructFields::None,
            });
        };
        ParseResult::Ok(PStruct { name, generics: args, fields })
    }
}

impl ParsableWith for PConstruct {
    type Data = Node<PIdent>;
    fn parse(ctx: &mut ParserCtx, name_node: Self::Data) -> ParseResult<Self> {
        let next = ctx.expect_peek()?;
        // TODO: this is not correct span; type should also span generics, which aren't even in
        // here yet
        let span = name_node.origin;
        let name = Node::new(
            PType {
                name: name_node,
                args: Vec::new(),
            },
            span,
        );
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
