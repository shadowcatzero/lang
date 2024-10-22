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
    DoubleDot,
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
    Bang,
    Ampersand,
    DoubleAmpersand,
    Pipe,
    DoublePipe,
    Comma,
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
            '!' => Self::Bang,
            '&' => Self::Ampersand,
            '|' => Self::Pipe,
            ',' => Self::Comma,
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
            },
            Self::Slash => match next {
                '/' => Self::DoubleSlash,
                _ => return,
            },
            Self::Ampersand => match next {
                '&' => Self::DoubleAmpersand,
                _ => return,
            },
            Self::Pipe => match next {
                '&' => Self::DoublePipe,
                _ => return,
            },
            Self::Dot => match next {
                '.' => Self::DoubleDot,
                _ => return,
            }
            _ => return,
        };
        cursor.advance();
    }
    pub fn str(&self) -> &str {
        match self {
            Self::Semicolon => ";",
            Self::Colon => ":",
            Self::DoubleColon => "::",
            Self::Equals => "=",
            Self::DoubleEquals => "==",
            Self::Arrow => "->",
            Self::DoubleArrow => "=>",
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Asterisk => "*",
            Self::Slash => "/",
            Self::DoubleSlash => "//",
            Self::Dot => ".",
            Self::DoubleDot => "..",
            Self::OpenParen => "(",
            Self::CloseParen => ")",
            Self::OpenCurly => "{",
            Self::CloseCurly => "}",
            Self::OpenSquare => "[",
            Self::CloseSquare => "]",
            Self::OpenAngle => "<",
            Self::CloseAngle => ">",
            Self::SingleQuote => "'",
            Self::DoubleQuote => "\"",
            Self::Bang => "!",
            Self::Comma => ",",
            Self::Ampersand => "&",
            Self::DoubleAmpersand => "&&",
            Self::Pipe => "|",
            Self::DoublePipe => "||",
        }
    }
}

impl Debug for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}'", self.str())
    }
}
