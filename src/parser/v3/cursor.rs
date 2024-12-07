use crate::ir::FilePos;

use super::error::ParserMsg;
use super::token::{CharCursor, Keyword, Symbol, Token, TokenInstance};

pub struct TokenCursor<'a> {
    cursor: CharCursor<'a>,
    next: Option<TokenInstance>,
    next_start: FilePos,
    prev_end: FilePos,
}

impl<'a> TokenCursor<'a> {
    pub fn next(&mut self) -> Option<TokenInstance> {
        self.prev_end = self
            .next
            .as_ref()
            .map(|i| i.span.end)
            .unwrap_or(FilePos::start());
        let next = TokenInstance::parse(&mut self.cursor);
        self.next_start = next
            .as_ref()
            .map(|i| i.span.end)
            .unwrap_or(FilePos::start());
        std::mem::replace(&mut self.next, next)
    }
    pub fn expect_next(&mut self) -> Result<TokenInstance, ParserMsg> {
        self.peek().ok_or(ParserMsg::unexpected_end())?;
        Ok(self.next().unwrap())
    }
    pub fn expect_token(&mut self, t: Token) -> Result<(), ParserMsg> {
        let next = self.expect_next()?;
        if t == next.token {
            Ok(())
        } else {
            Err(ParserMsg::unexpected_token(&next, &format!("{t:?}")))
        }
    }
    pub fn expect_sym(&mut self, symbol: Symbol) -> Result<(), ParserMsg> {
        self.expect_token(Token::Symbol(symbol))
    }
    pub fn next_on_new_line(&mut self) -> bool {
        self.next_start.line != self.prev_end.line
    }
    pub fn seek_sym(&mut self, sym: Symbol) {
        while self.next().is_some_and(|n| !n.is_symbol(sym)) {}
    }
    pub fn seek_syms(&mut self, syms: &[Symbol]) {
        while self
            .peek()
            .is_some_and(|n| !syms.iter().any(|s| n.is_symbol(*s)))
        {
            self.next();
        }
    }
    pub fn seek_sym_on_line(&mut self, sym: Symbol) {
        while !self.next_on_new_line() && self.next().is_some_and(|n| !n.is_symbol(sym)) {}
    }
    pub fn seek(&mut self, f: impl Fn(&TokenInstance) -> bool) -> Option<&TokenInstance> {
        loop {
            if f(self.peek()?) {
                return self.peek();
            }
            self.next();
        }
    }
    pub fn expect_kw(&mut self, kw: Keyword) -> Result<(), ParserMsg> {
        self.expect_token(Token::Keyword(kw))
    }
    pub fn peek(&self) -> Option<&TokenInstance> {
        self.next.as_ref()
    }
    pub fn expect_peek(&mut self) -> Result<&TokenInstance, ParserMsg> {
        self.peek().ok_or(ParserMsg::unexpected_end())
    }
    pub fn chars(&mut self) -> &mut CharCursor<'a> {
        &mut self.cursor
    }
    pub fn prev_end(&self) -> FilePos {
        self.prev_end
    }
    pub fn next_start(&self) -> FilePos {
        self.next_start
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
            next_start: FilePos::start(),
            prev_end: FilePos::start(),
        }
    }
}
