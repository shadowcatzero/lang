use std::fmt::Debug;

use super::{Ident, MaybeParsable, Node, Parsable, ParseResult, ParserMsg, Symbol, Token, Type};

pub struct VarDef {
    pub name: Node<Ident>,
    pub ty: Option<Node<Type>>,
}

impl Parsable for VarDef {
    fn parse(
        cursor: &mut super::TokenCursor,
        errors: &mut super::ParserOutput,
    ) -> ParseResult<Self> {
        let name = Node::parse(cursor, errors)?;
        if cursor.peek().is_some_and(|n| n.is_symbol(Symbol::Colon)) {
            cursor.next();
            Node::parse(cursor, errors).map(|ty| Self { name, ty: Some(ty) })
        } else {
            ParseResult::Ok(Self { name, ty: None })
        }
    }
}

pub struct SelfVar {
    pub ty: SelfType,
}

#[derive(PartialEq)]
pub enum SelfType {
    Ref,
    Take,
}

impl MaybeParsable for SelfVar {
    fn maybe_parse(
        cursor: &mut super::TokenCursor,
        errors: &mut super::ParserOutput,
    ) -> Result<Option<Self>, super::ParserMsg> {
        if let Some(mut next) = cursor.peek() {
            let mut ty = SelfType::Take;
            if next.is_symbol(Symbol::Ampersand) {
                cursor.next();
                ty = SelfType::Ref;
                next = cursor.expect_peek()?;
            }
            if let Token::Word(name) = &next.token {
                if name == "self" {
                    cursor.next();
                    return Ok(Some(Self { ty }));
                }
            }
            if ty != SelfType::Take {
                return Err(ParserMsg::unexpected_token(next, "self"));
            }
        }
        Ok(None)
    }
}

impl Debug for VarDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)?;
        if let Some(ty) = &self.ty {
            write!(f, ": {:?}", ty)?;
        }
        Ok(())
    }
}

impl Debug for SelfVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self.ty {
                SelfType::Ref => "&self",
                SelfType::Take => "self",
            }
        )
    }
}
