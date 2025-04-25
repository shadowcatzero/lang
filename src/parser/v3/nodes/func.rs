use super::{
    util::parse_list, PBlock, PIdent, Node, Parsable, ParseResult, ParserCtx,
    Symbol, PType, PVarDef,
};
use std::fmt::Debug;

pub struct PFunctionHeader {
    pub name: Node<PIdent>,
    pub args: Vec<Node<PVarDef>>,
    pub ret: Option<Node<PType>>,
}

pub struct PFunction {
    pub header: Node<PFunctionHeader>,
    pub body: Node<PBlock>,
}

impl Parsable for PFunctionHeader {
    fn parse(ctx: &mut ParserCtx) -> ParseResult<Self> {
        let name = ctx.parse()?;
        ctx.expect_sym(Symbol::OpenParen)?;
        // let sel = ctx.maybe_parse();
        // if sel.is_some() {
        //     if let Err(err) = ctx.expect_sym(Symbol::Comma) {
        //         ctx.err(err);
        //         ctx.seek_syms(&[Symbol::Comma, Symbol::CloseParen]);
        //         if ctx.peek().is_some_and(|i| i.is_symbol(Symbol::Comma)) {
        //             ctx.next();
        //         }
        //     }
        // }
        let args = parse_list(ctx, Symbol::CloseParen)?;
        let ret = if ctx.peek().is_some_and(|i| i.is_symbol(Symbol::Arrow)) {
            ctx.next();
            Some(ctx.parse()?)
        } else {
            None
        };
        ParseResult::Ok(Self {
            name,
            args,
            ret,
        })
    }
}

impl Parsable for PFunction {
    fn parse(ctx: &mut ParserCtx) -> ParseResult<Self> {
        let header = ctx.parse()?;
        ctx.expect_sym(Symbol::OpenCurly)?;
        let body = ctx.parse_with(Some(Symbol::CloseCurly))?;
        ParseResult::Ok(Self { header, body })
    }
}

impl Debug for PFunctionHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("fn ")?;
        self.name.fmt(f)?;
        f.write_str("(")?;
        // if let Some(s) = &self.sel {
        //     s.fmt(f)?;
        //     if self.args.first().is_some() {
        //         f.write_str(", ")?;
        //     }
        // }
        if let Some(a) = self.args.first() {
            a.fmt(f)?;
        }
        for arg in self.args.iter().skip(1) {
            f.write_str(", ")?;
            arg.fmt(f)?;
        }
        f.write_str(")")?;
        if let Some(ret) = &self.ret {
            write!(f, " -> {:?}", ret)?;
        }
        Ok(())
    }
}
impl Debug for PFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.header.fmt(f)?;
        f.write_str(" ")?;
        self.body.fmt(f)?;
        Ok(())
    }
}
