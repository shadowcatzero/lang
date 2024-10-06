use std::fmt::Debug;

use super::CharCursor;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Symbol {
    Semicolon,
    Colon,
    DoubleColon,
    Equals,
    DoubleEquals,
    Arrow,
    DoubleArrow,
    Plus,
    Minus,
    Asterisk,
    Slash,
    DoubleSlash,
    Dot,
    OpenParen,
    CloseParen,
    OpenCurly,
    CloseCurly,
    OpenSquare,
    CloseSquare,
    OpenAngle,
    CloseAngle,
    SingleQuote,
    DoubleQuote,
}

impl Symbol {
    pub fn parse(cursor: &mut CharCursor) -> Option<Self> {
        Self::from_char(cursor.peek()?).map(|mut s| {
            cursor.advance();
            s.finish(cursor);
            s
        })
    }
    pub fn from_char(c: char) -> Option<Self> {
        Some(match c {
            '(' => Self::OpenParen,
            ')' => Self::CloseParen,
            '[' => Self::OpenSquare,
            ']' => Self::CloseSquare,
            '{' => Self::OpenCurly,
            '}' => Self::CloseCurly,
            '<' => Self::OpenAngle,
            '>' => Self::CloseAngle,
            ';' => Self::Semicolon,
            ':' => Self::Colon,
            '+' => Self::Plus,
            '-' => Self::Minus,
            '*' => Self::Asterisk,
            '/' => Self::Slash,
            '=' => Self::Equals,
            '.' => Self::Dot,
            '\'' => Self::SingleQuote,
            '"' => Self::DoubleQuote,
            _ => return None,
        })
    }
    pub fn finish(&mut self, cursor: &mut CharCursor) {
        let Some(next) = cursor.peek() else {
            return;
        };
        *self = match self {
            Self::Colon => match next {
                ':' => Self::DoubleColon,
                _ => return,
            },
            Self::Minus => match next {
                '>' => Self::Arrow,
                _ => return,
            },
            Self::Equals => match next {
                '=' => Self::DoubleEquals,
                '>' => Self::DoubleArrow,
                _ => return,
            }
            Self::Slash => match next {
                '/' => Self::DoubleSlash,
                _ => return,
            }
            _ => return,
        };
        cursor.advance();
    }
    pub fn str(&self) -> &str {
        match self {
            Symbol::Semicolon => ";",
            Symbol::Colon => ":",
            Symbol::DoubleColon => "::",
            Symbol::Equals => "=",
            Symbol::DoubleEquals => "==",
            Symbol::Arrow => "->",
            Symbol::DoubleArrow => "=>",
            Symbol::Plus => "+",
            Symbol::Minus => "-",
            Symbol::Asterisk => "*",
            Symbol::Slash => "/",
            Symbol::DoubleSlash => "//",
            Symbol::Dot => ".",
            Symbol::OpenParen => "(",
            Symbol::CloseParen => ")",
            Symbol::OpenCurly => "{",
            Symbol::CloseCurly => "}",
            Symbol::OpenSquare => "[",
            Symbol::CloseSquare => "]",
            Symbol::OpenAngle => "<",
            Symbol::CloseAngle => ">",
            Symbol::SingleQuote => "'",
            Symbol::DoubleQuote => "\"",

        }
    }
}

impl Debug for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}'", self.str())
    }
}
