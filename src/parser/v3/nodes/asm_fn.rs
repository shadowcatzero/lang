use crate::compiler::riscv64::{AsmInstruction, Reg};

use super::{
    util::parse_list, AsmBlock, Ident, Keyword, Node, Parsable, ParseResult, SelfVar, Symbol,
};

#[derive(Debug)]
pub struct AsmFunctionHeader {
    pub name: Node<Ident>,
    pub sel: Option<Node<SelfVar>>,
    pub args: Vec<Node<Reg>>,
}

#[derive(Debug)]
pub struct AsmFunction {
    pub header: Node<AsmFunctionHeader>,
    pub body: Node<AsmBlock>,
}

impl Parsable for AsmFunctionHeader {
    fn parse(
        cursor: &mut super::TokenCursor,
        output: &mut super::ParserOutput,
    ) -> ParseResult<Self> {
        cursor.expect_kw(Keyword::Asm)?;
        cursor.expect_kw(Keyword::Fn)?;
        let name = Node::parse(cursor, output)?;
        cursor.expect_sym(Symbol::OpenParen)?;
        let sel = Node::maybe_parse(cursor, output);
        if sel.is_some() {
            if let Err(err) = cursor.expect_sym(Symbol::Comma) {
                output.err(err);
                cursor.seek_syms(&[Symbol::Comma, Symbol::CloseParen]);
                if cursor.peek().is_some_and(|i| i.is_symbol(Symbol::Comma)) {
                    cursor.next();
                }
            }
        }
        let args = parse_list(cursor, output, Symbol::CloseParen)?;
        ParseResult::Ok(Self { name, sel, args })
    }
}

impl Parsable for AsmFunction {
    fn parse(
        cursor: &mut super::TokenCursor,
        output: &mut super::ParserOutput,
    ) -> ParseResult<Self> {
        let header = Node::parse(cursor, output)?;
        let body = Node::parse(cursor, output)?;
        ParseResult::Ok(Self { header, body })
    }
}
