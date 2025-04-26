use std::{iter::Peekable, str::Chars};

use crate::common::FileID;

use super::super::{CompilerMsg, FilePos};

pub struct CharCursor<'a> {
    file: FileID,
    chars: Peekable<Chars<'a>>,
    next_pos: FilePos,
    prev_pos: FilePos,
}

impl<'a> CharCursor<'a> {
    pub fn next(&mut self) -> Option<char> {
        let res = self.peek()?;
        self.advance();
        Some(res)
    }
    pub fn expect(&mut self, c: char) -> Result<(), CompilerMsg> {
        let next = self.expect_next()?;
        if next == c {
            Ok(())
        } else {
            Err(CompilerMsg::at(
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
        self.prev_pos = self.next_pos;
        if next == '\n' {
            self.next_pos.col = 0;
            self.next_pos.line += 1;
        } else {
            self.next_pos.col += 1;
        }
    }
    pub fn expect_next(&mut self) -> Result<char, CompilerMsg> {
        self.next().ok_or(CompilerMsg::unexpected_end())
    }
    pub fn next_pos(&self) -> FilePos {
        self.next_pos
    }
    pub fn prev_pos(&self) -> FilePos {
        self.prev_pos
    }
    pub fn from_file_str(file: FileID, value: &'a str) -> Self {
        Self {
            chars: value.chars().peekable(),
            next_pos: FilePos::start(file),
            prev_pos: FilePos::start(file),
            file,
        }
    }
    pub fn file(&self) -> FileID {
        self.file
    }
}
