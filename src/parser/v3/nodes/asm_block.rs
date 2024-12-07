use super::{
    util::parse_list, PIdent, Node, Parsable, ParseResult, PInstruction, ParserCtx, Symbol,
};

pub struct PAsmBlock {
    pub instructions: Vec<Node<PInstruction>>,
    pub args: Vec<Node<PAsmBlockArg>>,
}

pub struct PAsmBlockArg {
    pub reg: Node<PIdent>,
    pub var: Node<PIdent>,
}

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

impl Parsable for PAsmBlockArg {
    fn parse(ctx: &mut ParserCtx) -> ParseResult<Self> {
        let reg = ctx.parse()?;
        ctx.expect_sym(Symbol::Equals)?;
        let var = ctx.parse()?;
        ParseResult::Ok(Self { reg, var })
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
