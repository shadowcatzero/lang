use super::{
    CharCursor, MaybeParsable, ParserError, ParserErrors, Symbol, Token, TokenCursor,
};
use std::fmt::Debug;

#[derive(Clone, PartialEq, Eq)]
pub enum Literal {
    String(String),
    Char(char),
    Number(Number),
    Unit,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Number {
    pub whole: String,
    pub decimal: Option<String>,
    pub ty: Option<String>,
}

impl MaybeParsable for Literal {
    fn maybe_parse(
        cursor: &mut TokenCursor,
        _: &mut ParserErrors,
    ) -> Result<Option<Self>, ParserError> {
        let inst = cursor.expect_peek()?;
        let mut res = match &inst.token {
            Token::Symbol(Symbol::SingleQuote) => {
                let chars = cursor.chars();
                let c = chars.expect_next()?;
                chars.expect('\'')?;
                Self::Char(c)
            }
            Token::Symbol(Symbol::DoubleQuote) => Self::String(string_from(cursor.chars())?),
            Token::Ident(text) => {
                let first = text.chars().next().unwrap();
                if first.is_ascii_digit() {
                    Self::Number(Number {
                        whole: text.to_string(),
                        decimal: None,
                        ty: None,
                    })
                } else {
                    return Ok(None);
                }
            }
            _ => return Ok(None),
        };
        cursor.next();
        if let (Some(next), Self::Number(num)) = (cursor.peek(), &mut res) {
            if next.token.is_symbol(Symbol::Dot) {
                cursor.next();
                if let Some(next) = cursor.peek() {
                    if let Token::Ident(i) = &next.token {
                        if i.chars().next().unwrap().is_ascii_digit() {
                            num.decimal = Some(i.to_string());
                            cursor.next();
                        }
                    }
                }
            }
        }
        Ok(Some(res))
    }
}
pub fn string_from(cursor: &mut CharCursor) -> Result<String, ParserError> {
    let mut str = String::new();
    loop {
        let c = cursor.expect_next()?;
        if c == '"' {
            return Ok(str);
        }
        str.push(match c {
            '\\' => {
                let next = cursor.expect_next()?;
                match next {
                    '"' => '"',
                    '\'' => '\'',
                    't' => '\t',
                    'n' => '\n',
                    '0' => '\0',
                    _ => {
                        todo!();
                    }
                }
            }
            _ => c,
        })
    }
}

impl Debug for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(str) => str.fmt(f),
            Self::Char(c) => c.fmt(f),
            Self::Number(n) => n.fmt(f),
            Self::Unit => f.write_str("()"),
        }
    }
}

impl Debug for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.whole)?;
        if let Some(d) = &self.decimal {
            write!(f, ".{}", d)?;
        }
        if let Some(ty) = &self.ty {
            write!(f, "T{}", ty)?;
        }
        Ok(())
    }
}
