use super::CharCursor;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StringType {
    DoubleQuote,
    SingleQuote,
}

impl StringType {
    pub fn from_start(c: char) -> Option<Self> {
        Some(match c {
            '"' => Self::DoubleQuote,
            '\'' => Self::SingleQuote,
            _ => return None,
        })
    }
    pub fn end(&self) -> char {
        match self {
            StringType::DoubleQuote => '"',
            StringType::SingleQuote => '\'',
        }
    }
    pub fn parse(
        &self,
        stream: &mut CharCursor,
    ) -> Result<String, String> {
        let end = self.end();
        let mut str = String::new();
        loop {
            let c = stream.expect_next()?;
            if c == end {
                return Ok(str);
            }
            str.push(match c {
                '\\' => {
                    let next = stream.expect_next()?;
                    match next {
                        '"' => '"',
                        '\'' => '\'',
                        't' => '\t',
                        'n' => '\n',
                        '0' => '\0',
                        c => return Err(format!("Unknown escape character {c}")),
                    }
                }
                _ => c,
            })
        }
    }
}
