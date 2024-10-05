use crate::token::{Keyword, Symbol, Token, TokenInstance};

use super::error::{unexpected_end, unexpected_token, ParserError};

pub struct TokenCursor<'a> {
    tokens: &'a [TokenInstance],
    pos: usize,
}

impl TokenCursor<'_> {
    pub fn next(&mut self) -> Option<&TokenInstance> {
        let res = self.tokens.get(self.pos);
        self.pos += 1;
        res
    }
    pub fn expect_next(&mut self) -> Result<&TokenInstance, ParserError> {
        self.next().ok_or(unexpected_end())
    }
    pub fn expect_token(&mut self, t: Token) -> Result<(), ParserError> {
        let next = self.expect_next()?;
        if t == next.token {
            Ok(())
        } else {
            unexpected_token(next, &format!("{t:?}"))
        }
    }
    pub fn expect_sym(&mut self, symbol: Symbol) -> Result<(), ParserError> {
        self.expect_token(Token::Symbol(symbol))
    }
    pub fn expect_kw(&mut self, kw: Keyword) -> Result<(), ParserError> {
        self.expect_token(Token::Keyword(kw))
    }
    pub fn peek(&self) -> Option<&TokenInstance> {
        self.tokens.get(self.pos)
    }
    pub fn expect_peek(&mut self) -> Result<&TokenInstance, ParserError> {
        self.peek().ok_or(unexpected_end())
    }
    pub fn expect_ident(&mut self) -> Result<String, ParserError> {
        let i = self.expect_next()?;
        let Token::Ident(n) = &i.token else {
            return unexpected_token(i, "an identifier");
        };
        Ok(n.to_string())
    }
}

impl<'a> From<&'a [TokenInstance]> for TokenCursor<'a> {
    fn from(tokens: &'a [TokenInstance]) -> Self {
        Self {
            tokens,
            pos: 0,
        }
    }
}
