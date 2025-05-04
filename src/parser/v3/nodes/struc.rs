use std::fmt::{Debug, Write};

use crate::{parser::ParsableWith, util::Padder};

use super::{
    util::parse_list, CompilerMsg, Node, PFieldDef, PGenericDef, PIdent, PType, PVarDef, Parsable,
    ParseResult, ParserCtx, Symbol,
};

#[derive(Debug)]
pub struct PStruct {
    pub name: Node<PIdent>,
    pub generics: Vec<Node<PGenericDef>>,
    pub fields: PStructFields,
}

pub struct PMap(pub Vec<Node<PFieldDef>>);

#[derive(Debug)]
pub enum PStructFields {
    Named(Vec<Node<PVarDef>>),
    Tuple(Vec<Node<PType>>),
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
        ParseResult::Ok(PStruct {
            name,
            generics: args,
            fields,
        })
    }
}

impl Parsable for PMap {
    fn parse(ctx: &mut ParserCtx) -> ParseResult<Self> {
        ctx.expect_sym(Symbol::OpenCurly);
        ParseResult::Ok(Self(parse_list(ctx, Symbol::CloseCurly)?))
    }
}

impl Debug for PMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{")?;
        let mut padder = Padder::new(f);
        for arg in &self.0 {
            padder.write_str(&format!("{arg:?},\n"))?;
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}
