use std::{collections::HashMap, path::PathBuf};

pub type FileID = usize;
pub type FileMap = HashMap<FileID, SrcFile>;

#[derive(Debug, Clone)]
pub struct SrcFile {
    pub path: PathBuf,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilePos {
    pub file: FileID,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct FileSpan {
    pub file: FileID,
    pub start: FilePos,
    pub end: FilePos,
}

impl FilePos {
    pub fn start(file: FileID) -> Self {
        Self {
            line: 0,
            col: 0,
            file,
        }
    }
}

impl FilePos {
    pub fn to(self, end: FilePos) -> FileSpan {
        FileSpan {
            start: self,
            end,
            file: self.file,
        }
    }
    pub fn char_span(self) -> FileSpan {
        FileSpan::at(self)
    }
}

const BEFORE: usize = 1;
const AFTER: usize = 0;

impl FileSpan {
    const BUILTIN_FILE: usize = usize::MAX;
    pub fn at(pos: FilePos) -> Self {
        Self {
            start: pos,
            end: pos,
            file: pos.file,
        }
    }
    pub fn builtin() -> Self {
        let pos = FilePos {
            file: Self::BUILTIN_FILE,
            line: 0,
            col: 0,
        };
        Self::at(pos)
    }
    pub fn is_builtin(&self) -> bool {
        self.file == Self::BUILTIN_FILE
    }
    pub fn write_for(&self, writer: &mut impl std::io::Write, file: &str) -> std::io::Result<()> {
        if self.is_builtin() {
            return Ok(());
        }
        let start = self.start.line.saturating_sub(BEFORE);
        let num_before = self.start.line - start;
        let mut lines = file.lines().skip(start);
        let width = format!("{}", self.end.line + AFTER).len();
        let same_line = self.start.line == self.end.line;
        for i in 0..num_before {
            writeln!(writer, "{:>width$} | {}", start + i, lines.next().unwrap())?;
        }
        let line = lines.next().unwrap();
        writeln!(writer, "{:>width$} | {}", self.start.line, line)?;
        let len = if same_line {
            self.end.col - self.start.col + 1
        } else {
            line.len() - self.start.col
        };
        writeln!(
            writer,
            "{} | {}",
            " ".repeat(width),
            " ".repeat(self.start.col) + &"^".repeat(len)
        )?;
        if !same_line {
            for _ in 0..self.end.line - self.start.line - 1 {
                lines.next();
            }
            let line = lines.next().unwrap();
            writeln!(writer, "{:>width$} | {}", self.end.line, line)?;
            writeln!(
                writer,
                "{} | {}",
                " ".repeat(width),
                "^".repeat(self.end.col + 1)
            )?;
        }
        // for i in 0..AFTER {
        //     if let Some(next) = lines.next() {
        //         writeln!(writer, "{:>width$} | {}", self.end.line + i + 1, next)?;
        //     }
        // }
        Ok(())
    }
}
