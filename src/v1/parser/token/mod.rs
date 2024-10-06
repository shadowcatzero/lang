mod cursor;
mod file;
mod keyword;
mod symbol;

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

#[derive(Debug)]
pub struct TokenInstance {
    pub token: Token,
    pub region: FileRegion,
}

impl TokenInstance {
    pub fn parse(cursor: &mut CharCursor) -> Option<TokenInstance> {
        cursor.skip_whitespace();
        cursor.peek()?;
        let start = cursor.pos();
        if let Some(s) = Symbol::parse(cursor) {
            if s == Symbol::DoubleSlash {
                while cursor.next() != Some('\n') {}
                return Self::parse(cursor);
            }
            let end = cursor.prev_pos();
            return Some(Self {
                token: Token::Symbol(s),
                region: FileRegion { start, end },
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
            region: FileRegion { start, end },
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
    pub fn is_keyword(&self, kw: Keyword) -> bool {
        match self {
            Token::Keyword(k) => *k == kw,
            _ => false,
        }
    }
}

impl TokenInstance {
    pub fn is_keyword(&self, kw: Keyword) -> bool {
        self.token.is_keyword(kw)
    }
    pub fn is_symbol(&self, symbol: Symbol) -> bool {
        self.token.is_symbol(symbol)
    }
}
