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
    Dot,
    OpenParen,
    CloseParen,
    OpenCurly,
    CloseCurly,
    OpenSquare,
    CloseSquare,
    OpenAngle,
    CloseAngle,
}

impl Symbol {
    pub fn from_start(c: char, stream: &mut CharCursor) -> Option<Result<Self, String>> {
        Some(Ok(match c {
            '(' => Self::OpenParen,
            ')' => Self::CloseParen,
            '[' => Self::OpenSquare,
            ']' => Self::CloseSquare,
            '{' => Self::OpenCurly,
            '}' => Self::CloseCurly,
            '<' => Self::OpenAngle,
            '>' => Self::CloseAngle,
            ';' => Self::Semicolon,
            ':' => {
                if stream.advance_if(':') {
                    Self::DoubleColon
                } else {
                    Self::Colon
                }
            }
            '+' => Self::Plus,
            '-' => {
                if stream.advance_if('>') {
                    Self::Arrow
                } else {
                    Self::Minus
                }
            }
            '*' => Self::Asterisk,
            '/' => Self::Slash,
            '=' => {
                if stream.advance_if('=') {
                    Self::DoubleEquals
                } else if stream.advance_if('>') {
                    Self::DoubleArrow
                } else {
                    Self::Equals
                }
            }
            '.' => Self::Dot,
            _ => return None,
        }))
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
            Symbol::Dot => ".",
            Symbol::OpenParen => "(",
            Symbol::CloseParen => ")",
            Symbol::OpenCurly => "{",
            Symbol::CloseCurly => "}",
            Symbol::OpenSquare => "[",
            Symbol::CloseSquare => "]",
            Symbol::OpenAngle => "<",
            Symbol::CloseAngle => ">",
        }
    }
}

impl Debug for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}'", self.str())
    }
}
