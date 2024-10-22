use std::fmt::Debug;

use super::{
    util::parse_list, Ident, Keyword, Node, Parsable, ParseResult, ParserMsg, ParserOutput,
    Symbol, TokenCursor, Type, VarDef,
};

#[derive(Debug)]
pub struct Struct {
    pub name: Node<Ident>,
    pub fields: StructFields,
}

#[derive(Debug)]
pub enum StructFields {
    Named(Vec<Node<VarDef>>),
    Tuple(Vec<Node<Type>>),
    None,
}

impl Parsable for Struct {
    fn parse(cursor: &mut TokenCursor, errors: &mut ParserOutput) -> ParseResult<Self> {
        cursor.expect_kw(Keyword::Struct)?;
        let name = Node::parse(cursor, errors)?;
        let next = cursor.expect_peek()?;
        let fields = if next.is_symbol(Symbol::Semicolon) {
            cursor.next();
            StructFields::None
        } else if next.is_symbol(Symbol::OpenCurly) {
            cursor.next();
            StructFields::Named(parse_list(cursor, errors, Symbol::CloseCurly)?)
        } else if next.is_symbol(Symbol::OpenParen) {
            cursor.next();
            StructFields::Tuple(parse_list(cursor, errors, Symbol::CloseParen)?)
        } else {
            errors.err(ParserMsg::unexpected_token(next, "`;`, `(`, or `{`"));
            return ParseResult::Recover(Struct {
                name,
                fields: StructFields::None,
            });
        };
        ParseResult::Ok(Struct { name, fields })
    }
}

