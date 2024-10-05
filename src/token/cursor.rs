#[derive(Debug, Clone, Copy)]
pub struct FilePos {
    pub line: usize,
    pub col: usize,
}

pub struct CharCursor<'a> {
    chars: &'a [u8],
    i: usize,
    pos: FilePos,
    prev_pos: FilePos,
}

// TODO: support unicode
impl CharCursor<'_> {
    pub fn next(&mut self) -> Option<char> {
        let res = self.get(self.i)?;
        self.mov();
        Some(res)
    }
    pub fn next_with_pos(&mut self) -> Option<(FilePos, char)> {
        let res = self.get(self.i)?;
        let pos = self.pos;
        self.mov();
        Some((pos, res))
    }
    pub fn peek(&mut self) -> Option<char> {
        self.get(self.i)
    }
    fn mov(&mut self) {
        self.prev_pos = self.pos;
        if self.chars[self.i] == b'\n' {
            self.pos.col = 0;
            self.pos.line += 1;
        } else {
            self.pos.col += 1;
        }
        self.i += 1;
    }
    pub fn advance_if(&mut self, c: char) -> bool {
        if let Some(c2) = self.get(self.i) {
            if c2 == c {
                self.mov();
                return true;
            }
        }
        false
    }
    pub fn expect_next(&mut self) -> Result<char, String> {
        self.next().ok_or("Unexpected end of input".to_string())
    }
    pub fn get(&self, i: usize) -> Option<char> {
        self.chars.get(i).map(|b| *b as char)
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
            chars: value.as_bytes(),
            i: 0,
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
