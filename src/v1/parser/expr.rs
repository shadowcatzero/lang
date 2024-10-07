use std::fmt::{Debug, Write};

use super::token::{Symbol, Token};
use super::{Body, Node, Parsable, ParserError, ParserErrors, TokenCursor, Val};

pub type ExprNode = Node<Box<Expr>>;

#[derive(Clone)]
pub enum Expr {
    Val(Node<Val>),
    Ident(String),
    BinaryOp(Operator, ExprNode, ExprNode),
    Block(Node<Body>),
    Call(ExprNode, Vec<Node<Expr>>),
    Group(ExprNode),
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
    Assign,
}

impl Expr {
    pub fn ended_with_error(&self) -> bool {
        match self {
            Expr::Val(_) => false,
            Expr::Ident(_) => false,
            Expr::BinaryOp(_, _, e) => e.is_err() || e.as_ref().is_ok_and(|e| e.ended_with_error()),
            Expr::Block(b) => b.is_err(),
            Expr::Call(_, _) => false,
            Expr::Group(_) => false,
        }
    }
}

impl Parsable for Expr {
    fn parse(cursor: &mut TokenCursor, errors: &mut ParserErrors) -> Result<Self, ParserError> {
        let start = cursor.next_pos();
        let next = cursor.expect_peek()?;
        let mut e1 = if next.is_symbol(Symbol::OpenParen) {
            cursor.next();
            if cursor.expect_peek()?.is_symbol(Symbol::CloseParen) {
                cursor.next();
                return Ok(Expr::Val(Node::new(
                    Val::Unit,
                    cursor.next_pos().char_span(),
                )));
            }
            let expr = Node::parse(cursor, errors).bx();
            if expr.is_err() {
                cursor.seek_sym(Symbol::CloseParen);
            }
            cursor.expect_sym(Symbol::CloseParen)?;
            Self::Group(expr)
        } else if next.is_symbol(Symbol::OpenCurly) {
            Self::Block(Node::parse(cursor, errors))
        } else if let Some(val) = Node::maybe_parse(cursor, errors) {
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
                    return Err(ParserError::unexpected_token(next, "an expression"));
                }
            }
        };
        let Some(mut next) = cursor.peek() else {
            return Ok(e1);
        };
        while next.is_symbol(Symbol::OpenParen) {
            cursor.next();
            let inner = Node::parse(cursor, errors);
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
            let mut n2 = Node::parse(cursor, errors).bx();
            if let Ok(box Self::BinaryOp(op2, n21, n22)) = n2.as_ref() {
                if op.presedence() > op2.presedence() {
                    n1 = Node::new(
                        Box::new(Self::BinaryOp(op, n1, n21.clone())),
                        start.to(n21.span.end),
                    );
                    op = *op2;
                    n2 = n22.clone();
                }
            }
            Self::BinaryOp(op, n1, n2)
        } else {
            e1
        })
    }
}

impl Operator {
    pub fn presedence(&self) -> u32 {
        match self {
            Operator::Assign => 0,
            Operator::LessThan => 1,
            Operator::GreaterThan => 1,
            Operator::Add => 2,
            Operator::Sub => 3,
            Operator::Mul => 4,
            Operator::Div => 5,
            Operator::Access => 6,
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
            Self::Assign => "=",
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
            Symbol::Equals => Operator::Assign,
            _ => {
                return None;
            }
        })
    }
    pub fn pad(&self) -> bool {
        match self {
            Self::Add => true,
            Self::Sub => true,
            Self::Mul => true,
            Self::Div => true,
            Self::LessThan => true,
            Self::GreaterThan => true,
            Self::Access => false,
            Self::Assign => true,
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
            Expr::Group(inner) => inner.fmt(f)?,
        }
        Ok(())
    }
}
