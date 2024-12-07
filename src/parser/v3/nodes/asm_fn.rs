use super::{
    util::parse_list, PAsmBlock, PIdent, Keyword, Node, Parsable, ParseResult, ParserCtx, Symbol, PType, PVarDef,
};

// #[derive(Debug)]
// pub struct AsmFunctionHeader {
//     pub name: Node<Ident>,
//     pub args: Vec<Node<AsmVarDef>>,
// }
//
// #[derive(Debug)]
// pub struct AsmFunction {
//     pub header: Node<AsmFunctionHeader>,
//     pub body: Node<AsmBlock>,
// }
//
// impl Parsable for AsmFunctionHeader {
//     fn parse(ctx: &mut ParserCtx) -> ParseResult<Self> {
//         ctx.expect_kw(Keyword::Asm)?;
//         ctx.expect_kw(Keyword::Fn)?;
//         let name = ctx.parse()?;
//         ctx.expect_sym(Symbol::OpenParen)?;
//         let args = parse_list(ctx, Symbol::CloseParen)?;
//         ParseResult::Ok(Self { name, args })
//     }
// }
//
// impl Parsable for AsmFunction {
//     fn parse(ctx: &mut ParserCtx) -> ParseResult<Self> {
//         let header = ctx.parse()?;
//         let body = ctx.parse()?;
//         ParseResult::Ok(Self { header, body })
//     }
// }
//
// pub struct AsmVarDef {
//     pub reg: Node<Ident>,
//     pub name: Node<Ident>,
//     pub ty: Option<Node<Type>>,
// }
//
// impl Parsable for AsmVarDef {
//     fn parse(ctx: &mut ParserCtx) -> ParseResult<Self> {
//         let reg = ctx.parse()?;
//         let name = ctx.parse()?;
//         if ctx.peek().is_some_and(|n| n.is_symbol(Symbol::Colon)) {
//             ctx.next();
//             ctx.parse().map(|ty| Self { reg, name, ty: Some(ty) })
//         } else {
//             ParseResult::Ok(Self { reg, name, ty: None })
//         }
//     }
// }
//
// impl std::fmt::Debug for AsmVarDef {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         self.name.fmt(f)?;
//         if let Some(ty) = &self.ty {
//             write!(f, ": {:?}", ty)?;
//         }
//         Ok(())
//     }
// }
