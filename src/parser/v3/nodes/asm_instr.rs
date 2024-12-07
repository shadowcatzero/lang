use super::{PIdent, Node, Parsable, ParseResult, ParserCtx, ParserMsg, Symbol};

pub struct PInstruction {
    pub op: Node<PIdent>,
    pub args: Vec<Node<PAsmArg>>,
}

pub enum PAsmArg {
    Value(Node<PIdent>),
    Ref(Node<PIdent>),
}

impl Parsable for PInstruction {
    fn parse(ctx: &mut ParserCtx) -> ParseResult<Self> {
        let op = ctx.parse()?;
        let mut args = Vec::new();
        if !ctx.next_on_new_line() {
            let arg = ctx.parse()?;
            args.push(arg);
            loop {
                if ctx.next_on_new_line() {
                    break;
                }
                ctx.expect_sym(Symbol::Comma)?;
                let arg = ctx.parse()?;
                args.push(arg);
            }
        }
        ParseResult::Ok(Self { op, args })
    }
}

impl Parsable for PAsmArg {
    fn parse(ctx: &mut ParserCtx) -> ParseResult<Self> {
        if let Some(ident) = ctx.maybe_parse() {
            return ParseResult::Ok(Self::Value(ident));
        }

        let next = ctx.expect_peek()?;
        if !next.is_symbol(Symbol::OpenCurly) {
            return ParseResult::Err(ParserMsg::unexpected_token(
                next,
                "An identifier or {identifier}",
            ));
        }
        ctx.next();

        let res = ctx.parse();
        if res.recover {
            ctx.seek_sym(Symbol::CloseCurly);
            return ParseResult::SubErr;
        }

        ctx.expect_sym(Symbol::CloseCurly)?;
        ParseResult::Ok(Self::Ref(res.node))
    }
}

impl std::fmt::Debug for PAsmArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Value(v) => v.fmt(f),
            Self::Ref(r) => write!(f, "{{{:?}}}", r),
        }
    }
}

impl std::fmt::Debug for PInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.op.fmt(f)?;
        let mut iter = self.args.iter();
        if let Some(arg) = iter.next() {
            f.write_str(" ")?;
            arg.fmt(f)?;
        }
        for arg in iter {
            f.write_str(", ")?;
            arg.fmt(f)?;
        }
        Ok(())
    }
}
