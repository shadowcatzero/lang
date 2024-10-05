use std::fmt::{Debug, Write};

use super::{
    cursor::TokenCursor,
    error::{unexpected_token, ParserError},
    Body,
};
use crate::token::{StringType, Symbol, Token, TokenInstance};

pub enum Expr {
    Const(ConstVal),
    Ident(String),
    Op(Operator, Vec<Expr>),
    Block(Body),
    Call(Box<Expr>, Vec<Expr>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    LessThan,
    GreaterThan,
    Offset,
}

#[derive(PartialEq, Eq)]
pub enum ConstVal {
    String(String),
    Char(char),
    Number(String),
    Unit,
}

impl Expr {
    pub fn parse(cursor: &mut TokenCursor) -> Result<Self, ParserError> {
        let Some(next) = cursor.peek() else {
            return Ok(Expr::Const(ConstVal::Unit));
        };
        let mut cur = if next.is_symbol(Symbol::OpenParen) {
            cursor.next();
            let expr = Self::parse(cursor)?;
            cursor.expect_sym(Symbol::CloseParen)?;
            expr
        } else if next.is_symbol(Symbol::OpenCurly) {
            let expr = Body::parse(cursor)?;
            Expr::Block(expr)
        } else {
            let unit = Self::parse_unit(next)?;
            cursor.next();
            unit
        };
        let Some(mut next) = cursor.peek() else {
            return Ok(cur);
        };
        while next.is_symbol(Symbol::OpenParen) {
            cursor.next();
            let inner = Self::parse(cursor)?;
            cursor.expect_sym(Symbol::CloseParen)?;
            cur = Self::Call(Box::new(cur), vec![inner]);
            let Some(next2) = cursor.peek() else {
                return Ok(cur);
            };
            next = next2
        }
        if let Some(op) = Operator::from_token(&next.token) {
            cursor.next();
            let next = Self::parse(cursor)?;
            let mut vals = vec![cur];
            if let Self::Op(op_next, mut vs) = next {
                if op == op_next {
                    vals.extend(vs);
                } else if op.presedence() > op_next.presedence() {
                    vals.push(vs.remove(0));
                    if vs.len() == 1 {
                        return Ok(Self::Op(
                            op_next,
                            vec![Self::Op(op, vals), vs.pop().unwrap()],
                        ));
                    } else {
                        vals.push(Self::Op(op_next, vs));
                    }
                } else {
                    vals.push(Self::Op(op_next, vs));
                }
            } else {
                vals.push(next);
            }
            return Ok(Self::Op(op, vals));
        };
        match next.token {
            Token::Symbol(Symbol::Semicolon | Symbol::CloseParen | Symbol::CloseCurly) => Ok(cur),
            _ => unexpected_token(next, "an operator or ending"),
        }
    }
    fn parse_unit(inst: &TokenInstance) -> Result<Self, ParserError> {
        match &inst.token {
            Token::String(ty, s) => {
                Self::parse_str(*ty, s).map_err(|e| ParserError::from_instances(&[inst], e))
            }
            Token::Ident(name) => Ok(Self::parse_ident(name.to_string())),
            _ => unexpected_token(inst, "a string or a name"),
        }
    }
    fn parse_str(ty: StringType, s: &str) -> Result<Self, String> {
        match ty {
            StringType::DoubleQuote => Ok(Self::Const(ConstVal::String(s.to_string()))),
            StringType::SingleQuote => {
                if s.len() == 1 {
                    Ok(Self::Const(ConstVal::Char(s.chars().next().unwrap())))
                } else {
                    Err("Characters must only have one char".to_string())
                }
            }
        }
    }
    fn parse_ident(str: String) -> Self {
        match str.chars().next().unwrap() {
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | '.' => {
                Self::Const(ConstVal::Number(str))
            }
            _ => Self::Ident(str),
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
            Operator::Offset => 5,
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
            Self::Offset => ".",
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
            Symbol::Dot => Operator::Offset,
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
            Operator::Offset => false,
        }
    }
}

impl Debug for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Const(c) => c.fmt(f),
            Expr::Ident(n) => f.write_str(n),
            Expr::Block(b) => b.fmt(f),
            Expr::Op(op, exprs) => {
                f.write_char('(')?;
                exprs[0].fmt(f)?;
                for expr in exprs.iter().skip(1) {
                    if op.pad() {
                        write!(f, " {} ", op.str())?;
                    } else {
                        f.write_str(op.str())?;
                    }
                    expr.fmt(f)?;
                }
                f.write_char(')')?;
                Ok(())
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
                Ok(())
            }
        }
    }
}

impl Debug for ConstVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(str) => str.fmt(f),
            Self::Char(c) => c.fmt(f),
            Self::Number(str) => f.write_str(str),
            Self::Unit => f.write_str("()"),
        }
    }
}
