use super::{
    Function, Keyword, Node, Parsable, ParseResult, ParserError, ParserErrors, TokenCursor,
};
use std::fmt::Debug;

pub struct Module {
    pub functions: Vec<Node<Function>>,
}

impl Parsable for Module {
    fn parse(cursor: &mut TokenCursor, errors: &mut ParserErrors) -> ParseResult<Self> {
        let mut functions = Vec::new();
        loop {
            let Some(next) = cursor.peek() else {
                return ParseResult::Ok(Self { functions });
            };
            if next.is_keyword(Keyword::Fn) {
                let res = Node::parse(cursor, errors);
                functions.push(res.node);
                if res.recover {
                    return ParseResult::Recover(Self { functions });
                }
            } else {
                errors.add(ParserError::unexpected_token(next, "fn"));
                cursor.next();
            }
        }
    }
}

impl Debug for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.functions.fmt(f)
    }
}
