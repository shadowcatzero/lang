use std::{collections::HashSet, iter::Peekable, str::Chars};

use super::{error::ParserError, util::WHITESPACE_SET};

#[derive(Debug, Clone, Copy)]
pub struct FilePos {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct FileRegion {
    pub start: FilePos,
    pub end: FilePos,
}

pub struct CharCursor<'a> {
    chars: Peekable<Chars<'a>>,
    pos: FilePos,
    prev_pos: FilePos,
}

impl CharCursor<'_> {
    pub fn until(&mut self, set: &HashSet<char>) -> String {
        let mut str = String::new();
        loop {
            let Some(next) = self.peek() else {
                return str;
            };
            if set.contains(&next) {
                return str;
            }
            str.push(next);
            self.advance();
        }
    }
    pub fn skip_whitespace(&mut self) {
        while self.peek().is_some_and(|c| c.is_whitespace()) {
            self.advance();
        }
        let mut copy = self.chars.clone();
        if let Some('/') = copy.next() {
            if let Some('/') = copy.next() {
                self.advance();
                self.advance();
                while self.next() != Some('\n') {}
                self.skip_whitespace();
            }
        }
    }
    pub fn next(&mut self) -> Option<char> {
        let res = self.peek()?;
        self.advance();
        Some(res)
    }
    pub fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }
    pub fn advance(&mut self) {
        self.prev_pos = self.pos;
        if self.peek().is_some_and(|c| c == '\n') {
            self.pos.col = 0;
            self.pos.line += 1;
        } else {
            self.pos.col += 1;
        }
        self.chars.next();
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
    pub fn advance_if_str(&mut self, exp: &str, end: &HashSet<char>) -> bool {
        let mut new = self.chars.clone();
        for e in exp.chars() {
            let Some(c) = new.next() else {
                return false;
            };
            if e != c {
                return false;
            }
        }
        if new.peek().is_some_and(|c| !end.contains(c)) {
            return false;
        }
        for _ in 0..exp.len() {
            self.advance();
        }
        true
    }
    pub fn expect_char(&mut self, c: char) -> Result<(), ParserError> {
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
    pub fn expect_next(&mut self) -> Result<char, ParserError> {
        self.next().ok_or(ParserError::unexpected_end())
    }
    pub fn expect_peek(&mut self) -> Result<char, ParserError> {
        self.peek().ok_or(ParserError::unexpected_end())
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

impl FilePos {
    pub fn start() -> Self {
        Self { line: 0, col: 0 }
    }
}
