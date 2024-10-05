use crate::token::{Keyword, Symbol};
use std::fmt::Debug;

mod body;
mod cursor;
mod error;
mod expr;
pub use body::*;
pub use cursor::*;
pub use expr::*;
pub use error::*;

#[derive(Debug)]
pub struct Module {
    functions: Vec<Function>,
}

pub struct Function {
    pub name: String,
    pub body: Body,
}

impl Module {
    pub fn parse(cursor: &mut TokenCursor) -> Result<Self, ParserError> {
        let mut functions = Vec::new();
        loop {
            let Some(next) = cursor.peek() else {
                return Ok(Self { functions });
            };
            if next.is_keyword(Keyword::Fn) {
                functions.push(Function::parse(cursor)?);
            } else {
                return unexpected_token(cursor.next().unwrap(), "fn");
            }
        }
    }
}

impl Function {
    pub fn parse(cursor: &mut TokenCursor) -> Result<Self, ParserError> {
        cursor.expect_kw(Keyword::Fn)?;
        let name = cursor.expect_ident()?;
        cursor.expect_sym(Symbol::OpenParen)?;
        cursor.expect_sym(Symbol::CloseParen)?;
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
