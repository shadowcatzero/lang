use std::fmt::Debug;
use super::{Parsable, ParseResult, ParserMsg, Token};

pub struct Ident(String);

impl Ident {
    pub fn val(&self) -> &String {
        &self.0
    }
}

impl Parsable for Ident {
    fn parse(cursor: &mut super::TokenCursor, errors: &mut super::ParserOutput) -> ParseResult<Self> {
        let next = cursor.expect_peek()?;
        let Token::Word(name) = &next.token else {
            return ParseResult::Err(ParserMsg::unexpected_token(next, "an identifier"));
        };
        let name = name.to_string();
        cursor.next();
        ParseResult::Ok(Self(name))
    }
}

impl Debug for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

