use super::{Symbol, Token};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InfixOp {
    Add,
    Sub,
    Mul,
    Div,
    LessThan,
    GreaterThan,
    Assign,
}

impl InfixOp {
    pub fn precedence(&self) -> u32 {
        match self {
            Self::Assign => 0,
            Self::LessThan => 1,
            Self::GreaterThan => 1,
            Self::Add => 2,
            Self::Sub => 3,
            Self::Mul => 4,
            Self::Div => 5,
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
            Self::Assign => "=",
        }
    }
    pub fn pad(&self) -> bool {
        match self {
            Self::Add => true,
            Self::Sub => true,
            Self::Mul => true,
            Self::Div => true,
            Self::LessThan => true,
            Self::GreaterThan => true,
            Self::Assign => true,
        }
    }
}

pub enum PostfixOp {
    Not,
    Ref,
    Deref,
}

impl InfixOp {
    pub fn from_token(token: &Token) -> Option<Self> {
        let Token::Symbol(symbol) = token else {
            return None;
        };
        Some(match symbol {
            Symbol::OpenAngle => Self::LessThan,
            Symbol::CloseAngle => Self::GreaterThan,
            Symbol::Plus => Self::Add,
            Symbol::Minus => Self::Sub,
            Symbol::Asterisk => Self::Mul,
            Symbol::Slash => Self::Div,
            Symbol::Equals => Self::Assign,
            _ => {
                return None;
            }
        })
    }
}

impl PostfixOp {
    pub fn str(&self) -> &str {
        match self {
            Self::Not => "!",
            Self::Ref => "@",
            Self::Deref => "*",
        }
    }
    pub fn from_token(token: &Token) -> Option<Self> {
        let Token::Symbol(symbol) = token else {
            return None;
        };
        Some(match symbol {
            Symbol::At => Self::Ref,
            Symbol::Bang => Self::Not,
            Symbol::Asterisk => Self::Deref,
            _ => {
                return None;
            }
        })
    }
}
