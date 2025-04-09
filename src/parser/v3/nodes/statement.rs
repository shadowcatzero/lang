use super::{Keyword, Node, PExpr, PVarDef, Parsable, ParseResult, ParserCtx, Symbol, Token};

pub enum PStatement {
    Let(Node<PVarDef>, Node<PExpr>),
    Return(Option<Node<PExpr>>),
    Expr(Node<PExpr>),
}

impl Parsable for PStatement {
    fn parse(ctx: &mut ParserCtx) -> ParseResult<Self> {
        let next = ctx.expect_peek()?;
        match next.token {
            Token::Keyword(Keyword::Let) => {
                ctx.next();
                let def = ctx.parse()?;
                ctx.expect_sym(Symbol::Equals)?;
                ctx.parse().map(|expr| Self::Let(def, expr))
            }
            Token::Keyword(Keyword::Return) => {
                ctx.next();
                if ctx.peek().is_some_and(|t| t.is_symbol(Symbol::Semicolon)) {
                    ParseResult::Ok(Self::Return(None))
                } else {
                    ctx.parse().map(|res| Self::Return(Some(res)))
                }
            }
            _ => ctx.parse().map(Self::Expr),
        }
    }
}

impl std::fmt::Debug for PStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PStatement::Let(n, e) => {
                f.write_str("let ")?;
                n.fmt(f)?;
                f.write_str(" = ")?;
                e.fmt(f)?;
            }
            PStatement::Return(e) => {
                f.write_str("return ")?;
                e.fmt(f)?;
            }
            PStatement::Expr(e) => {
                e.fmt(f)?;
            }
        }
        Ok(())
    }
}
