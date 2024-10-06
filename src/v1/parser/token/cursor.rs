use std::{iter::Peekable, str::Chars};

use crate::v1::parser::ParserError;

use super::FilePos;

pub struct CharCursor<'a> {
    chars: Peekable<Chars<'a>>,
    pos: FilePos,
    prev_pos: FilePos,
}

impl CharCursor<'_> {
    pub fn next(&mut self) -> Option<char> {
        let res = self.peek()?;
        self.advance();
        Some(res)
    }
    pub fn expect(&mut self, c: char) -> Result<(), ParserError> {
        let next = self.expect_next()?;
        if next == c {
            Ok(())
        } else {
            Err(ParserError::at(
                self.prev_pos,
                format!("unexpected char '{next}'; expected '{c}'"),
            ))
        }
    }
    pub fn skip_whitespace(&mut self) {
        while self.peek().is_some_and(|c| c.is_whitespace()) {
            self.advance();
        }
    }
    pub fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }
    pub fn advance(&mut self) {
        let Some(next) = self.chars.next() else {
            return;
        };
        self.prev_pos = self.pos;
        if next == '\n' {
            self.pos.col = 0;
            self.pos.line += 1;
        } else {
            self.pos.col += 1;
        }
    }
    pub fn advance_if(&mut self, c: char) -> bool {
        if let Some(c2) = self.peek() {
            if c2 == c {
                self.advance();
                return true;
            }
        }
        false
    }
    pub fn expect_next(&mut self) -> Result<char, ParserError> {
        self.next()
            .ok_or(ParserError::from_msg("Unexpected end of input".to_string()))
    }
    pub fn pos(&self) -> FilePos {
        self.pos
    }
    pub fn prev_pos(&self) -> FilePos {
        self.prev_pos
    }
}

impl<'a> From<&'a str> for CharCursor<'a> {
    fn from(value: &'a str) -> Self {
        Self {
            chars: value.chars().peekable(),
            pos: FilePos::start(),
            prev_pos: FilePos::start(),
        }
    }
}
