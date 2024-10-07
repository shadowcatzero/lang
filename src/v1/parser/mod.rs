use std::fmt::Debug;

mod body;
mod cursor;
mod error;
mod expr;
mod token;
mod val;
mod node;

pub use body::*;
pub use cursor::*;
pub use error::*;
pub use expr::*;
pub use val::*;
pub use node::*;

use token::*;

#[derive(Debug)]
pub struct Module {
    functions: Vec<Node<Function>>,
}

#[derive(Clone)]
pub struct Function {
    pub name: String,
    pub body: Node<Body>,
}

impl Parsable for Module {
    fn parse(cursor: &mut TokenCursor) -> Result<Self, ParserError> {
        let mut functions = Vec::new();
        loop {
            let Some(next) = cursor.peek() else {
                return Ok(Self { functions });
            };
            if next.is_keyword(Keyword::Fn) {
                functions.push(Node::parse(cursor));
            } else {
                return Err(ParserError::unexpected_token(next, "fn"));
            }
        }
    }
}

impl Parsable for Function {
    fn parse(cursor: &mut TokenCursor) -> Result<Self, ParserError> {
        cursor.expect_kw(Keyword::Fn)?;
        let name = cursor.expect_ident()?;
        cursor.expect_sym(Symbol::OpenParen)?;
        cursor.expect_sym(Symbol::CloseParen)?;
        let body = Node::parse(cursor);
        Ok(Self { name, body })
    }
}

impl Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("fn ")?;
        f.write_str(&self.name)?;
        f.write_str("() ")?;
        self.body.fmt(f)?;
        Ok(())
    }
}

impl NodeContainer for Module {
    fn children(&self) -> Vec<Node<Box<dyn NodeContainer>>> {
        self.functions.iter().map(|f| f.containerr()).collect()
    }
}

impl NodeContainer for Function {
    fn children(&self) -> Vec<Node<Box<dyn NodeContainer>>> {
        vec![self.body.containerr()]
    }
}
