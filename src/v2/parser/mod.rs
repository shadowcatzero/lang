use std::{collections::HashSet, fmt::Debug, sync::LazyLock};

mod body;
mod cursor;
mod error;
mod expr;
mod util;

pub use body::*;
pub use cursor::*;
pub use error::*;
pub use expr::*;
use util::WHITESPACE_SET;

#[derive(Debug)]
pub struct Module {
    functions: Vec<Function>,
}

pub struct Function {
    pub name: String,
    pub body: Body,
}

static NAME_END: LazyLock<HashSet<char>> = LazyLock::new(|| {
    let mut set = WHITESPACE_SET.clone();
    set.extend(&['(']);
    set
});

impl Module {
    pub fn parse(cursor: &mut CharCursor) -> Result<Self, ParserError> {
        let mut functions = Vec::new();
        loop {
            let next = cursor.until(&WHITESPACE_SET);
            if next.is_empty() {
                return Ok(Self { functions });
            }
            if next == "fn" {
                functions.push(Function::parse(cursor)?);
            } else {
                return Err(ParserError::at(cursor.pos(), "expected fn".to_string()));
            }
        }
    }
}

impl Function {
    pub fn parse(cursor: &mut CharCursor) -> Result<Self, ParserError> {
        cursor.skip_whitespace();
        let name = cursor.until(&NAME_END);
        if name.is_empty() {
            return Err(ParserError::at(cursor.pos(), "expected function name".to_string()));
        }
        cursor.expect_char('(')?;
        cursor.expect_char(')')?;
        let body = Body::parse(cursor)?;
        Ok(Self { name, body })
    }
}

impl Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("fn ")?;
        f.write_str(&self.name)?;
        f.write_str("() ")?;
        self.body.fmt(f)?;
        Ok(())
    }
}
