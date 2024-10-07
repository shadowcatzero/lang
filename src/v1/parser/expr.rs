use std::fmt::{Debug, Write};

use super::token::{Symbol, Token};
use super::{Body, Node, NodeContainer, Parsable, ParserError, TokenCursor, Val};

#[derive(Clone)]
pub enum Expr {
    Val(Node<Val>),
    Ident(String),
    BinaryOp(Operator, Node<Box<Expr>>, Node<Box<Expr>>),
    Block(Node<Body>),
    Call(Node<Box<Expr>>, Vec<Node<Expr>>),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    LessThan,
    GreaterThan,
    Access,
}

impl Parsable for Expr {
    fn parse(cursor: &mut TokenCursor) -> Result<Self, ParserError> {
        let start = cursor.next_pos();
        let Some(next) = cursor.peek() else {
            return Ok(Expr::Val(Node::new(
                Val::Unit,
                cursor.next_pos().char_span(),
            )));
        };
        let mut e1 = if next.is_symbol(Symbol::OpenParen) {
            cursor.next();
            let expr = Node::parse(cursor);
            if expr.is_ok() {
                cursor.expect_sym(Symbol::CloseParen)?;
            } else {
                cursor.seek_sym(Symbol::CloseParen);
            }
            expr.take()?
        } else if next.is_symbol(Symbol::OpenCurly) {
            Self::Block(Node::parse(cursor))
        } else if let Some(val) = Node::maybe_parse(cursor) {
            Self::Val(val)
        } else {
            let next = cursor.peek().unwrap();
            match &next.token {
                Token::Ident(name) => {
                    let name = name.to_string();
                    cursor.next();
                    Self::Ident(name)
                }
                _ => {
                    return Ok(Expr::Val(Node::new(
                        Val::Unit,
                        cursor.next_pos().char_span(),
                    )))
                }
            }
        };
        let Some(mut next) = cursor.peek() else {
            return Ok(e1);
        };
        while next.is_symbol(Symbol::OpenParen) {
            cursor.next();
            let inner = Node::parse(cursor);
            cursor.expect_sym(Symbol::CloseParen)?;
            let end = cursor.prev_end();
            e1 = Self::Call(Node::new(Box::new(e1), start.to(end)), vec![inner]);
            let Some(next2) = cursor.peek() else {
                return Ok(e1);
            };
            next = next2
        }
        let end = cursor.prev_end();
        Ok(if let Some(mut op) = Operator::from_token(&next.token) {
            cursor.next();
            let mut n1 = Node::new(Box::new(e1), start.to(end));
            let mut n2 = Node::<Self>::parse(cursor);
            if let Ok(Self::BinaryOp(op2, n21, n22)) = &*n2 {
                if op.presedence() > op2.presedence() {
                    n1 = Node::new(
                        Box::new(Self::BinaryOp(op, n1, n21.clone())),
                        start.to(n21.span.end),
                    );
                    op = *op2;
                    n2 = n22.clone().unbx();
                }
            }
            Self::BinaryOp(op, n1, n2.bx())
        } else {
            e1
        })
    }
}

impl Operator {
    pub fn presedence(&self) -> u32 {
        match self {
            Operator::LessThan => 0,
            Operator::GreaterThan => 0,
            Operator::Add => 1,
            Operator::Sub => 2,
            Operator::Mul => 3,
            Operator::Div => 4,
            Operator::Access => 5,
        }
    }
    pub fn str(&self) -> &str {
        match self {
            Self::Add => "+",
            Self::Sub => "-",
            Self::Mul => "*",
            Self::Div => "/",
            Self::LessThan => "<",
            Self::GreaterThan => ">",
            Self::Access => ".",
        }
    }
    pub fn from_token(token: &Token) -> Option<Self> {
        let Token::Symbol(symbol) = token else {
            return None;
        };
        Some(match symbol {
            Symbol::OpenAngle => Operator::LessThan,
            Symbol::CloseAngle => Operator::GreaterThan,
            Symbol::Plus => Operator::Add,
            Symbol::Minus => Operator::Sub,
            Symbol::Asterisk => Operator::Mul,
            Symbol::Slash => Operator::Div,
            Symbol::Dot => Operator::Access,
            _ => {
                return None;
            }
        })
    }
    pub fn pad(&self) -> bool {
        match self {
            Operator::Add => true,
            Operator::Sub => true,
            Operator::Mul => true,
            Operator::Div => true,
            Operator::LessThan => true,
            Operator::GreaterThan => true,
            Operator::Access => false,
        }
    }
}

impl Debug for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Val(c) => c.fmt(f)?,
            Expr::Ident(n) => f.write_str(n)?,
            Expr::Block(b) => b.fmt(f)?,
            Expr::BinaryOp(op, e1, e2) => {
                write!(f, "({:?}", *e1)?;
                if op.pad() {
                    write!(f, " {} ", op.str())?;
                } else {
                    write!(f, "{}", op.str())?;
                }
                write!(f, "{:?})", *e2)?;
            }
            Expr::Call(n, args) => {
                n.fmt(f)?;
                f.write_char('(')?;
                if let Some(a) = args.first() {
                    a.fmt(f)?;
                }
                for arg in args.iter().skip(1) {
                    f.write_str(", ")?;
                    arg.fmt(f)?;
                }
                f.write_char(')')?;
            }
        }
        Ok(())
    }
}

impl NodeContainer for Expr {
    fn children(&self) -> Vec<Node<Box<dyn NodeContainer>>> {
        match self {
            Expr::Val(_) => Vec::new(),
            Expr::Ident(_) => Vec::new(),
            Expr::BinaryOp(_, e1, e2) => vec![e1.container(), e2.container()],
            Expr::Block(b) => vec![b.containerr()],
            Expr::Call(e1, rest) => [e1.container()]
                .into_iter()
                .chain(rest.iter().map(|e| e.containerr()))
                .collect(),
        }
    }
}
