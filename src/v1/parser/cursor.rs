use super::error::ParserError;
use super::token::{CharCursor, Keyword, Symbol, Token, TokenInstance};
use super::FilePos;

pub struct TokenCursor<'a> {
    cursor: CharCursor<'a>,
    next: Option<TokenInstance>,
    next_pos: FilePos,
    prev_end: FilePos,
}

impl<'a> TokenCursor<'a> {
    pub fn next(&mut self) -> Option<TokenInstance> {
        self.prev_end = self.cursor.prev_pos();
        self.next_pos = self.cursor.next_pos();
        std::mem::replace(&mut self.next, TokenInstance::parse(&mut self.cursor))
    }
    pub fn expect_next(&mut self) -> Result<TokenInstance, ParserError> {
        self.peek().ok_or(ParserError::unexpected_end())?;
        Ok(self.next().unwrap())
    }
    pub fn expect_token(&mut self, t: Token) -> Result<(), ParserError> {
        let next = self.expect_next()?;
        if t == next.token {
            Ok(())
        } else {
            Err(ParserError::unexpected_token(&next, &format!("{t:?}")))
        }
    }
    pub fn expect_sym(&mut self, symbol: Symbol) -> Result<(), ParserError> {
        self.expect_token(Token::Symbol(symbol))
    }
    pub fn seek_sym(&mut self, symbol: Symbol) {
        while self
            .next()
            .is_some_and(|n| n.token != Token::Symbol(symbol))
        {}
    }
    pub fn seek(&mut self, f: impl Fn(&TokenInstance) -> bool) -> Option<&TokenInstance> {
        loop {
            if f(self.peek()?) {
                return self.peek();
            }
            self.next();
        }
    }
    pub fn expect_kw(&mut self, kw: Keyword) -> Result<(), ParserError> {
        self.expect_token(Token::Keyword(kw))
    }
    pub fn peek(&self) -> Option<&TokenInstance> {
        self.next.as_ref()
    }
    pub fn expect_peek(&mut self) -> Result<&TokenInstance, ParserError> {
        self.peek().ok_or(ParserError::unexpected_end())
    }
    pub fn expect_ident(&mut self) -> Result<String, ParserError> {
        let i = self.expect_next()?;
        let Token::Ident(n) = &i.token else {
            return Err(ParserError::unexpected_token(&i, "an identifier"));
        };
        Ok(n.to_string())
    }
    pub fn chars(&mut self) -> &mut CharCursor<'a> {
        &mut self.cursor
    }
    pub fn prev_end(&self) -> FilePos {
        self.prev_end
    }
    pub fn next_pos(&self) -> FilePos {
        self.next_pos
    }
}

impl<'a> From<&'a str> for TokenCursor<'a> {
    fn from(string: &'a str) -> Self {
        Self::from(CharCursor::from(string))
    }
}

impl<'a> From<CharCursor<'a>> for TokenCursor<'a> {
    fn from(mut cursor: CharCursor<'a>) -> Self {
        let cur = TokenInstance::parse(&mut cursor);
        Self {
            cursor,
            next: cur,
            next_pos: FilePos::start(),
            prev_end: FilePos::start(),
        }
    }
}
