use super::{Symbol, Token};

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

