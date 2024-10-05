mod cursor;
mod keyword;
mod string;
mod symbol;

use cursor::*;
pub use keyword::*;
pub use string::*;
pub use symbol::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    String(StringType, String),
    Symbol(Symbol),
    Ident(String),
    Keyword(Keyword),
}

#[derive(Debug, Clone, Copy)]
pub struct FileRegion {
    pub start: FilePos,
    pub end: FilePos,
}

#[derive(Debug)]
pub struct TokenInstance {
    pub token: Token,
    pub loc: FileRegion,
}

pub fn parse(str: &str) -> Result<Vec<TokenInstance>, String> {
    let mut tokens = Vec::new();
    let mut word = String::new();
    let mut word_start = FilePos::start();
    let mut word_end = FilePos::start();
    let mut cursor = CharCursor::from(str);
    while let Some((start, c)) = cursor.next_with_pos() {
        if c == '/' && cursor.advance_if('/') {
            while cursor.peek() != Some('\n') {
                cursor.next();
            }
            continue;
        }
        let add = if c.is_whitespace() {
            None
        } else if let Some(lit) = StringType::from_start(c) {
            let str = lit.parse(&mut cursor)?;
            let end = cursor.prev_pos();
            Some(TokenInstance {
                token: Token::String(lit, str),
                loc: FileRegion { start, end },
            })
        } else if let Some(symbol) = Symbol::from_start(c, &mut cursor) {
            let end = cursor.prev_pos();
            Some(TokenInstance {
                token: Token::Symbol(symbol?),
                loc: FileRegion { start, end },
            })
        } else {
            word.push(c);
            word_end = start;
            continue;
        };
        if !word.is_empty() {
            tokens.push(TokenInstance {
                token: Token::from_string(&word),
                loc: FileRegion { start: word_start, end: word_end },
            });
            word.clear();
        }
        word_start = cursor.pos();
        if let Some(token) = add {
            tokens.push(token);
        }
    }
    if !word.is_empty() {
        tokens.push(TokenInstance {
            token: Token::from_string(&word),
            loc: FileRegion { start: word_start, end: word_end },
        });
    }
    Ok(tokens)
}

impl Token {
    fn from_string(str: &str) -> Self {
        match Keyword::from_string(str) {
            Some(k) => Self::Keyword(k),
            None => Self::Ident(str.to_string()),
        }
    }
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
