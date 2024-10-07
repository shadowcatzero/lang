#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilePos {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct FileSpan {
    pub start: FilePos,
    pub end: FilePos,
}

impl FilePos {
    pub fn start() -> Self {
        Self { line: 0, col: 0 }
    }
}

impl FilePos {
    pub fn to(self, end: FilePos) -> FileSpan {
        FileSpan { start: self, end }
    }
    pub fn char_span(self) -> FileSpan {
        FileSpan::at(self)
    }
}

const BEFORE: usize = 1;
const AFTER: usize = 1;

impl FileSpan {
    pub fn at(pos: FilePos) -> Self {
        Self {
            start: pos,
            end: pos,
        }
    }
    pub fn write_for(&self, writer: &mut impl std::io::Write, file: &str) -> std::io::Result<()> {
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
        for i in 0..AFTER {
            if let Some(next) = lines.next() {
                writeln!(writer, "{:>width$} | {}", self.end.line + i + 1, next)?;
            }
        }
        Ok(())
    }
}
