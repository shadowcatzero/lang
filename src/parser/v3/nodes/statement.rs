use super::{PExpr, Keyword, Node, Parsable, ParseResult, ParserCtx, Symbol, Token, PVarDef};

pub enum PStatement {
    Let(Node<PVarDef>, Node<PExpr>),
    Return(Node<PExpr>),
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
                ctx.parse().map(Self::Return)
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
