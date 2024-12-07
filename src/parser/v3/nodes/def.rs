use std::fmt::Debug;

use super::{
    PIdent, MaybeParsable, Node, Parsable, ParseResult, ParserCtx, ParserMsg, Symbol, Token, PType,
};

pub struct PVarDef {
    pub name: Node<PIdent>,
    pub ty: Option<Node<PType>>,
}

impl Parsable for PVarDef {
    fn parse(ctx: &mut ParserCtx) -> ParseResult<Self> {
        let name = ctx.parse()?;
        if ctx.peek().is_some_and(|n| n.is_symbol(Symbol::Colon)) {
            ctx.next();
            ctx.parse().map(|ty| Self { name, ty: Some(ty) })
        } else {
            ParseResult::Ok(Self { name, ty: None })
        }
    }
}

pub struct SelfVar {
    pub ty: SelfType,
}

#[derive(PartialEq)]
pub enum SelfType {
    Ref,
    Take,
}

impl MaybeParsable for SelfVar {
    fn maybe_parse(ctx: &mut ParserCtx) -> Result<Option<Self>, super::ParserMsg> {
        if let Some(mut next) = ctx.peek() {
            let mut ty = SelfType::Take;
            if next.is_symbol(Symbol::Ampersand) {
                ctx.next();
                ty = SelfType::Ref;
                next = ctx.expect_peek()?;
            }
            if let Token::Word(name) = &next.token {
                if name == "self" {
                    ctx.next();
                    return Ok(Some(Self { ty }));
                }
            }
            if ty != SelfType::Take {
                return Err(ParserMsg::unexpected_token(next, "self"));
            }
        }
        Ok(None)
    }
}

impl Debug for PVarDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)?;
        if let Some(ty) = &self.ty {
            write!(f, ": {:?}", ty)?;
        }
        Ok(())
    }
}

impl Debug for SelfVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self.ty {
                SelfType::Ref => "&self",
                SelfType::Take => "self",
            }
        )
    }
}
