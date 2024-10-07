mod cursor;
mod file;
mod keyword;
mod symbol;

use std::ops::Deref;

pub use cursor::*;
pub use file::*;
pub use keyword::*;
pub use symbol::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    Symbol(Symbol),
    Ident(String),
    Keyword(Keyword),
}

#[derive(Clone)]
pub struct TokenInstance {
    pub token: Token,
    pub span: FileSpan,
}

impl TokenInstance {
    pub fn parse(cursor: &mut CharCursor) -> Option<TokenInstance> {
        cursor.skip_whitespace();
        cursor.peek()?;
        let start = cursor.next_pos();
        if let Some(s) = Symbol::parse(cursor) {
            if s == Symbol::DoubleSlash {
                while cursor.next() != Some('\n') {}
                return Self::parse(cursor);
            }
            let end = cursor.prev_pos();
            return Some(Self {
                token: Token::Symbol(s),
                span: FileSpan { start, end },
            });
        }
        let mut word = String::new();
        while let Some(c) = cursor.peek() {
            if c.is_whitespace() || Symbol::from_char(c).is_some() {
                break;
            }
            word.push(c);
            cursor.advance();
        }
        let end = cursor.prev_pos();
        let token = if let Some(keyword) = Keyword::from_string(&word) {
            Token::Keyword(keyword)
        } else {
            Token::Ident(word)
        };
        Some(Self {
            token,
            span: FileSpan { start, end },
        })
    }
}

impl Token {
    pub fn is_symbol(&self, symbol: Symbol) -> bool {
        match self {
            Token::Symbol(s) => *s == symbol,
            _ => false,
        }
    }
    pub fn is_symbol_and(&self, f: impl Fn(Symbol) -> bool) -> bool {
        match self {
            Token::Symbol(s) => f(*s),
            _ => false,
        }
    }
    pub fn is_keyword(&self, kw: Keyword) -> bool {
        match self {
            Token::Keyword(k) => *k == kw,
            _ => false,
        }
    }
}

impl Deref for TokenInstance {
    type Target = Token;

    fn deref(&self) -> &Self::Target {
        &self.token
    }
}
