use super::{
    util::parse_list, Node, PExpr, PIdent, PInstruction, Parsable, ParseResult, ParserCtx, Symbol,
};

pub struct PAsmBlock {
    pub instructions: Vec<Node<PInstruction>>,
    pub args: Vec<Node<PUAsmBlockArg>>,
}

pub enum PAsmBlockArg<R, V> {
    In { reg: R, var: V },
    Out { reg: R },
}

pub type PUAsmBlockArg = PAsmBlockArg<Node<PIdent>, Node<PExpr>>;

impl Parsable for PAsmBlock {
    fn parse(ctx: &mut ParserCtx) -> ParseResult<Self> {
        let args = if ctx.expect_peek()?.is_symbol(Symbol::OpenParen) {
            ctx.next();
            parse_list(ctx, Symbol::CloseParen)?
        } else {
            Vec::new()
        };
        ctx.expect_sym(Symbol::OpenCurly)?;
        let mut instructions = Vec::new();
        while !ctx.expect_peek()?.is_symbol(Symbol::CloseCurly) {
            let res = ctx.parse();
            instructions.push(res.node);
            if res.recover {
                ctx.seek_sym_on_line(Symbol::CloseCurly);
            }
        }
        ctx.expect_sym(Symbol::CloseCurly)?;
        ParseResult::Ok(Self { instructions, args })
    }
}

impl Parsable for PUAsmBlockArg {
    fn parse(ctx: &mut ParserCtx) -> ParseResult<Self> {
        let reg = ctx.parse::<PIdent>()?;
        ParseResult::Ok(if reg.inner.as_ref().is_some_and(|s| s.0 == "out") {
            ctx.expect_sym(Symbol::Equals)?;
            let reg = ctx.parse()?;
            Self::Out { reg }
        } else {
            ctx.expect_sym(Symbol::Equals)?;
            let var = ctx.parse()?;
            Self::In { reg, var }
        })
    }
}

impl std::fmt::Debug for PAsmBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("asm {")?;
        for i in &self.instructions {
            f.write_str("\n    ")?;
            i.fmt(f)?;
        }
        if !self.instructions.is_empty() {
            f.write_str("\n")?;
        }
        f.write_str("}")?;
        Ok(())
    }
}
