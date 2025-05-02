use crate::parser::{Parsable, ParseResult};

use super::{PString, ParserCtx, Symbol, Token};
use std::fmt::Debug;

#[derive(Clone, PartialEq, Eq)]
pub enum PLiteral {
    String(String),
    Char(char),
    Number(PNumber),
    Unit,
}

#[derive(Clone, PartialEq, Eq)]
pub struct PNumber {
    pub whole: String,
    pub decimal: Option<String>,
    pub ty: Option<String>,
}

impl Parsable for Option<PLiteral> {
    fn parse(ctx: &mut ParserCtx) -> ParseResult<Self> {
        let inst = ctx.expect_peek()?;
        ParseResult::Ok(Some(match &inst.token {
            Token::Symbol(Symbol::SingleQuote) => {
                let chars = ctx.chars();
                let c = chars.expect_next()?;
                chars.expect('\'')?;
                ctx.next();
                PLiteral::Char(c)
            }
            Token::Symbol(Symbol::DoubleQuote) => {
                ctx.next();
                let s = ctx.parse::<PString>()?;
                return match s.inner {
                    Some(s) => ParseResult::Ok(Some(PLiteral::String(s.0))),
                    None => ParseResult::SubErr,
                };
            }
            Token::Word(text) => {
                let first = text.chars().next().unwrap();
                if !first.is_ascii_digit() {
                    return ParseResult::Ok(None);
                }
                let (whole, ty) = parse_whole_num(text);
                let mut num = PNumber {
                    whole,
                    decimal: None,
                    ty,
                };
                ctx.next();
                if num.ty.is_none() && ctx.peek().is_some_and(|i| i.is_symbol(Symbol::Dot)) {
                    ctx.next();
                    if let Some(next) = ctx.peek() {
                        if let Token::Word(i) = &next.token {
                            if i.chars().next().unwrap().is_ascii_digit() {
                                let (decimal, ty) = parse_whole_num(i);
                                num.decimal = Some(decimal);
                                num.ty = ty;
                                ctx.next();
                            }
                        }
                    }
                }
                PLiteral::Number(num)
            }
            _ => return ParseResult::Ok(None),
        }))
    }
}

pub fn parse_whole_num(text: &str) -> (String, Option<String>) {
    let mut whole = String::new();
    let mut ty = String::new();
    for c in text.chars() {
        if ty.is_empty() {
            if c.is_ascii_digit() {
                whole.push(c);
            } else if c != '_' {
                ty.push(c);
            }
        } else {
            ty.push(c);
        }
    }
    (whole, if ty.is_empty() { None } else { Some(ty) })
}

impl Debug for PLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(str) => str.fmt(f),
            Self::Char(c) => c.fmt(f),
            Self::Number(n) => n.fmt(f),
            Self::Unit => f.write_str("()"),
        }
    }
}

impl Debug for PNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.whole)?;
        if let Some(d) = &self.decimal {
            write!(f, ".{}", d)?;
        }
        if let Some(ty) = &self.ty {
            write!(f, "_{}", ty)?;
        }
        Ok(())
    }
}
