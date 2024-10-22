use super::{util::{parse_list, parse_list_nosep}, Function, FunctionHeader, Ident, Keyword, Node, Parsable, Symbol, Type};

#[derive(Debug)]
pub struct Trait {
    pub name: Node<Ident>,
    pub fns: Vec<Node<FunctionHeader>>
}

#[derive(Debug)]
pub struct Impl {
    pub trait_: Node<Type>,
    pub for_: Node<Type>,
    pub fns: Vec<Node<Function>>
}

impl Parsable for Trait {
    fn parse(cursor: &mut super::TokenCursor, errors: &mut super::ParserOutput) -> super::ParseResult<Self> {
        cursor.expect_kw(Keyword::Trait)?;
        let name = Node::parse(cursor, errors)?;
        cursor.expect_sym(Symbol::OpenCurly)?;
        let fns = parse_list(cursor, errors, Symbol::CloseCurly)?;
        super::ParseResult::Ok(Self {name, fns})
    }
}

impl Parsable for Impl {
    fn parse(cursor: &mut super::TokenCursor, errors: &mut super::ParserOutput) -> super::ParseResult<Self> {
        cursor.expect_kw(Keyword::Impl)?;
        let trait_ = Node::parse(cursor, errors)?;
        cursor.expect_kw(Keyword::For)?;
        let for_ = Node::parse(cursor, errors)?;
        cursor.expect_sym(Symbol::OpenCurly)?;
        let fns = parse_list_nosep(cursor, errors, Symbol::CloseCurly)?;
        super::ParseResult::Ok(Self {trait_, for_, fns})
    }
}
