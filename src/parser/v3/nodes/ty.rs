use std::fmt::Debug;

use super::{util::parse_list, Node, Parsable, ParseResult, ParserMsg, Symbol, Token};

pub struct Type {
    pub name: String,
    pub args: Vec<Node<Type>>,
}

impl Type {
    pub fn unit() -> Self {
        Self {
            name: "()".to_string(),
            args: Vec::new(),
        }
    }
}

impl Parsable for Type {
    fn parse(
        cursor: &mut super::TokenCursor,
        errors: &mut super::ParserOutput,
    ) -> ParseResult<Self> {
        let next = cursor.expect_peek()?;
        let res = if next.is_symbol(Symbol::Ampersand) {
            cursor.next();
            let arg = Node::parse(cursor, errors)?;
            Self {
                name: "&".to_string(),
                args: vec![arg],
            }
        } else {
            let Token::Word(name) = &next.token else {
                return ParseResult::Err(ParserMsg::unexpected_token(next, "a type identifier"));
            };
            let n = name.to_string();
            cursor.next();
            let mut args = Vec::new();
            if let Some(next) = cursor.peek() {
                if next.is_symbol(Symbol::OpenAngle) {
                    cursor.next();
                    args = parse_list(cursor, errors, Symbol::CloseAngle)?;
                }
            }
            Self { name: n, args }
        };
        ParseResult::Ok(res)
    }
}

impl Debug for Type {
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
