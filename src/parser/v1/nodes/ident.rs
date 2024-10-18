use std::fmt::Debug;
use super::{Parsable, ParseResult, ParserError, Resolvable, Token};

pub struct Ident(String);

impl Ident {
    pub fn val(&self) -> &String {
        &self.0
    }
}

impl Parsable for Ident {
    fn parse(cursor: &mut super::TokenCursor, errors: &mut super::ParserErrors) -> ParseResult<Self> {
        let next = cursor.expect_peek()?;
        let Token::Ident(name) = &next.token else {
            return ParseResult::Err(ParserError::unexpected_token(next, "an identifier"));
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

impl Resolvable<Ident> for Ident {
    fn resolve(self) -> Result<Ident, ()> {
        Ok(self)
    }
}
