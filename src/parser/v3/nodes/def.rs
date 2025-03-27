use std::fmt::Debug;

use super::{
    CompilerMsg, MaybeParsable, Node, PExpr, PIdent, PType, Parsable, ParseResult, ParserCtx,
    Symbol, Token,
};

pub struct PVarDef {
    pub name: Node<PIdent>,
    pub ty: Option<Node<PType>>,
}

pub struct PFieldDef {
    pub name: Node<PIdent>,
    pub val: Option<Node<PExpr>>,
}

impl Parsable for PVarDef {
    fn parse(ctx: &mut ParserCtx) -> ParseResult<Self> {
        let name = ctx.parse()?;
        ParseResult::Ok(if ctx.peek().is_some_and(|n| n.is_symbol(Symbol::Colon)) {
            ctx.next();
            Self {
                name,
                ty: Some(ctx.parse()?),
            }
        } else {
            Self { name, ty: None }
        })
    }
}

impl Parsable for PFieldDef {
    fn parse(ctx: &mut ParserCtx) -> ParseResult<Self> {
        let name = ctx.parse()?;
        ParseResult::Ok(if ctx.peek().is_some_and(|n| n.is_symbol(Symbol::Colon)) {
            ctx.next();
            Self {
                name,
                val: Some(ctx.parse()?),
            }
        } else {
            Self { name, val: None }
        })
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
    fn maybe_parse(ctx: &mut ParserCtx) -> Result<Option<Self>, CompilerMsg> {
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
                return Err(CompilerMsg::unexpected_token(next, "self"));
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

impl Debug for PFieldDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)?;
        if let Some(val) = &self.val {
            write!(f, ": {:?}", val)?;
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
