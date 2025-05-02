use super::{CompilerMsg, Parsable, ParseResult, ParserCtx, Token};
use std::{
    fmt::{Debug, Display},
    ops::Deref,
};

#[derive(Clone)]
pub struct PIdent(pub String);

impl Parsable for PIdent {
    fn parse(ctx: &mut ParserCtx) -> ParseResult<Self> {
        let next = ctx.expect_peek()?;
        let Token::Word(name) = &next.token else {
            return ParseResult::Err(CompilerMsg::unexpected_token(next, "an identifier"));
        };
        let name = name.to_string();
        ctx.next();
        ParseResult::Ok(Self(name))
    }
}

impl Parsable for Option<PIdent> {
    fn parse(ctx: &mut ParserCtx) -> ParseResult<Self> {
        let Some(next) = ctx.peek() else {
            return ParseResult::Ok(None);
        };
        let Token::Word(name) = &next.token else {
            return ParseResult::Ok(None);
        };
        let name = name.to_string();
        ctx.next();
        ParseResult::Ok(Some(PIdent(name)))
    }
}

impl Debug for PIdent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for PIdent {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for PIdent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
