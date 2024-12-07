use super::{MaybeParsable, Parsable, ParseResult, ParserCtx, ParserMsg, Token};
use std::{
    fmt::{Debug, Display},
    ops::Deref,
};

#[derive(Clone)]
pub struct PIdent(String);

impl Parsable for PIdent {
    fn parse(ctx: &mut ParserCtx) -> ParseResult<Self> {
        let next = ctx.expect_peek()?;
        let Token::Word(name) = &next.token else {
            return ParseResult::Err(ParserMsg::unexpected_token(next, "an identifier"));
        };
        let name = name.to_string();
        ctx.next();
        ParseResult::Ok(Self(name))
    }
}

impl MaybeParsable for PIdent {
    fn maybe_parse(ctx: &mut ParserCtx) -> Result<Option<Self>, ParserMsg> {
        let Some(next) = ctx.peek() else { return Ok(None) };
        let Token::Word(name) = &next.token else {
            return Ok(None);
        };
        let name = name.to_string();
        ctx.next();
        Ok(Some(Self(name)))
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
