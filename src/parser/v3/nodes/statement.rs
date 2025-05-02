use super::{
    Keyword, Node, PExpr, PFunction, PIdent, PStruct, PVarDef, Parsable, ParseResult, ParserCtx,
    Symbol, Token,
};

pub enum PStatement {
    Let(Node<PVarDef>, Node<PExpr>),
    Return(Option<Node<PExpr>>),
    Expr(Node<PExpr>),
}

pub enum PConstStatement {
    Fn(Node<PFunction>),
    Struct(Node<PStruct>),
    Import(Node<PIdent>),
}

pub enum PStatementLike {
    Statement(PStatement),
    Const(PConstStatement),
}

impl Parsable for PStatementLike {
    fn parse(ctx: &mut ParserCtx) -> ParseResult<Self> {
        let next = ctx.expect_peek()?;
        match next.token {
            Token::Keyword(Keyword::Let) => {
                ctx.next();
                let def = ctx.parse()?;
                ctx.expect_sym(Symbol::Equals)?;
                ctx.parse()
                    .map_res(|expr| Self::Statement(PStatement::Let(def, expr)))
            }
            Token::Keyword(Keyword::Return) => {
                ctx.next();
                if ctx.peek().is_some_and(|t| t.is_symbol(Symbol::Semicolon)) {
                    ParseResult::Ok(Self::Statement(PStatement::Return(None)))
                } else {
                    ctx.parse()
                        .map_res(|res| Self::Statement(PStatement::Return(Some(res))))
                }
            }
            Token::Keyword(Keyword::Fn) => {
                ctx.next();
                ParseResult::Ok(Self::Const(PConstStatement::Fn(ctx.parse()?)))
            }
            Token::Keyword(Keyword::Struct) => {
                ctx.next();
                ParseResult::Ok(Self::Const(PConstStatement::Struct(ctx.parse()?)))
            }
            Token::Keyword(Keyword::Import) => {
                ctx.next();
                ParseResult::Ok(Self::Const(PConstStatement::Import(ctx.parse()?)))
            }
            _ => ctx.parse().map_res(|n| Self::Statement(PStatement::Expr(n))),
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
impl std::fmt::Debug for PConstStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fn(fun) => {
                fun.fmt(f)?;
            }
            Self::Struct(s) => {
                s.fmt(f)?;
            }
            Self::Import(s) => {
                writeln!(f, "import {:?}", s);
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for PStatementLike {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Statement(s) => s.fmt(f),
            Self::Const(c) => c.fmt(f),
        }
    }
}
