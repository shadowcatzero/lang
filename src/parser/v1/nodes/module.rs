use super::{
    Function, Keyword, MaybeResolved, Node, Parsable, ParseResult, ParserError, ParserErrors,
    Resolvable, Resolved, TokenCursor, Unresolved,
};
use std::fmt::Debug;

pub struct Module<R: MaybeResolved> {
    pub functions: Vec<Node<Function<R>, R>>,
}

impl Parsable for Module<Unresolved> {
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

impl Debug for Module<Unresolved> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.functions.fmt(f)
    }
}

impl Resolvable<Module<Resolved>> for Module<Unresolved> {
    fn resolve(self) -> Result<Module<Resolved>, ()> {
        Ok(Module {
            functions: self
                .functions
                .into_iter()
                .map(|f| f.resolve())
                .collect::<Result<_, _>>()?,
        })
    }
}
