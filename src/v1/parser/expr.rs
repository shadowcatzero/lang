use std::fmt::{Debug, Write};

use super::token::{Symbol, Token};
use super::{Body, Number, ParserError, TokenCursor, Val};

pub enum Expr {
    Val(Val),
    Ident(String),
    BinaryOp(Operator, Box<Expr>, Box<Expr>),
    Block(Body),
    Call(Box<Expr>, Vec<Expr>),
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

impl Expr {
    pub fn parse(cursor: &mut TokenCursor) -> Result<Self, ParserError> {
        let Some(next) = cursor.peek() else {
            return Ok(Expr::Val(Val::Unit));
        };
        let mut e1 = if next.is_symbol(Symbol::OpenParen) {
            cursor.next();
            let expr = Self::parse(cursor)?;
            cursor.expect_sym(Symbol::CloseParen)?;
            expr
        } else if next.is_symbol(Symbol::OpenCurly) {
            let expr = Body::parse(cursor)?;
            Expr::Block(expr)
        } else {
            Self::parse_unit(cursor)?
        };
        let Some(mut next) = cursor.peek() else {
            return Ok(e1);
        };
        while next.is_symbol(Symbol::OpenParen) {
            cursor.next();
            let inner = Self::parse(cursor)?;
            cursor.expect_sym(Symbol::CloseParen)?;
            e1 = Self::Call(Box::new(e1), vec![inner]);
            let Some(next2) = cursor.peek() else {
                return Ok(e1);
            };
            next = next2
        }
        Ok(if let Some(op) = Operator::from_token(&next.token) {
            cursor.next();
            let e2 = Self::parse(cursor)?;
            if let Self::BinaryOp(op_next, e3, e4) = e2 {
                if op.presedence() > op_next.presedence() {
                    Self::BinaryOp(op_next, Box::new(Self::BinaryOp(op, Box::new(e1), e3)), e4)
                } else {
                    Self::BinaryOp(op, Box::new(e1), Box::new(Self::BinaryOp(op_next, e3, e4)))
                }
            } else {
                Self::BinaryOp(op, Box::new(e1), Box::new(e2))
            }
        } else {
            e1
        })
    }
    fn parse_unit(cursor: &mut TokenCursor) -> Result<Self, ParserError> {
        if let Some(val) = Val::parse(cursor)? {
            return Ok(Self::Val(val));
        }
        let inst = cursor.expect_next()?;
        match &inst.token {
            Token::Ident(name) => Ok(Self::Ident(name.to_string())),
            _ => Err(ParserError::unexpected_token(
                &inst,
                "an identifier or value",
            )),
        }
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
