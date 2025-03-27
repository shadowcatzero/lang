use std::fmt::Debug;

use super::{
    util::parse_list, CompilerMsg, Node, PIdent, Parsable, ParseResult, ParserCtx, Symbol, Token,
};

pub struct PType {
    pub name: Node<PIdent>,
    pub args: Vec<Node<PType>>,
}

impl Parsable for PType {
    fn parse(ctx: &mut ParserCtx) -> ParseResult<Self> {
        let next = ctx.expect_peek()?;
        let res = if next.is_symbol(Symbol::Ampersand) {
            let name = Node::new(PIdent("&".to_string()), next.span);
            ctx.next();
            let arg = ctx.parse()?;
            Self {
                name,
                args: vec![arg],
            }
        } else {
            let n = ctx.parse()?;
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
        write!(f, "{:?}", self.name)?;
        if self.name.as_ref().is_some_and(|n| n.0 == "&") {
            write!(f, "{:?}", self.args[0])?;
        } else if !self.args.is_empty() {
            write!(f, "<{:?}>", self.args)?;
        }
        Ok(())
    }
}
