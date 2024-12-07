use super::{
    util::{parse_list, parse_list_nosep},
    PFunction, PFunctionHeader, PIdent, Keyword, Node, Parsable, ParserCtx, Symbol, PType,
};

#[derive(Debug)]
pub struct PTrait {
    pub name: Node<PIdent>,
    pub fns: Vec<Node<PFunctionHeader>>,
}

#[derive(Debug)]
pub struct PImpl {
    pub trait_: Node<PType>,
    pub for_: Node<PType>,
    pub fns: Vec<Node<PFunction>>,
}

impl Parsable for PTrait {
    fn parse(ctx: &mut ParserCtx) -> super::ParseResult<Self> {
        ctx.expect_kw(Keyword::Trait)?;
        let name = ctx.parse()?;
        ctx.expect_sym(Symbol::OpenCurly)?;
        let fns = parse_list(ctx, Symbol::CloseCurly)?;
        super::ParseResult::Ok(Self { name, fns })
    }
}

impl Parsable for PImpl {
    fn parse(ctx: &mut ParserCtx) -> super::ParseResult<Self> {
        ctx.expect_kw(Keyword::Impl)?;
        let trait_ = ctx.parse()?;
        ctx.expect_kw(Keyword::For)?;
        let for_ = ctx.parse()?;
        ctx.expect_sym(Symbol::OpenCurly)?;
        let fns = parse_list_nosep(ctx, Symbol::CloseCurly)?;
        super::ParseResult::Ok(Self { trait_, for_, fns })
    }
}
