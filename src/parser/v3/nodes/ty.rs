use std::fmt::Debug;

use super::{util::parse_list, Node, Parsable, ParseResult, ParserCtx, ParserMsg, Symbol, Token};

pub struct PType {
    pub name: String,
    pub args: Vec<Node<PType>>,
}

impl PType {
    pub fn unit() -> Self {
        Self {
            name: "()".to_string(),
            args: Vec::new(),
        }
    }
}

impl Parsable for PType {
    fn parse(ctx: &mut ParserCtx) -> ParseResult<Self> {
        let next = ctx.expect_peek()?;
        let res = if next.is_symbol(Symbol::Ampersand) {
            ctx.next();
            let arg = ctx.parse()?;
            Self {
                name: "&".to_string(),
                args: vec![arg],
            }
        } else {
            let Token::Word(name) = &next.token else {
                return ParseResult::Err(ParserMsg::unexpected_token(next, "a type identifier"));
            };
            let n = name.to_string();
            ctx.next();
            let mut args = Vec::new();
            if let Some(next) = ctx.peek() {
                if next.is_symbol(Symbol::OpenAngle) {
                    ctx.next();
                    args = parse_list(ctx, Symbol::CloseAngle)?;
                }
            }
            Self { name: n, args }
        };
        ParseResult::Ok(res)
    }
}

impl Debug for PType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)?;
        if self.name == "&" {
            write!(f, "{:?}", self.args[0])?;
        } else if !self.args.is_empty() {
            write!(f, "<{:?}>", self.args)?;
        }
        Ok(())
    }
}
