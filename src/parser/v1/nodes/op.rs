use super::{Symbol, Token};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    LessThan,
    GreaterThan,
    Access,
    Assign,
}

impl BinaryOperator {
    pub fn presedence(&self) -> u32 {
        match self {
            Self::Assign => 0,
            Self::LessThan => 1,
            Self::GreaterThan => 1,
            Self::Add => 2,
            Self::Sub => 3,
            Self::Mul => 4,
            Self::Div => 5,
            Self::Access => 6,
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
            Symbol::OpenAngle => Self::LessThan,
            Symbol::CloseAngle => Self::GreaterThan,
            Symbol::Plus => Self::Add,
            Symbol::Minus => Self::Sub,
            Symbol::Asterisk => Self::Mul,
            Symbol::Slash => Self::Div,
            Symbol::Dot => Self::Access,
            Symbol::Equals => Self::Assign,
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

pub enum UnaryOperator {
    Not,
    Ref,
}

impl UnaryOperator {
    pub fn str(&self) -> &str {
        match self {
            Self::Not => "!",
            Self::Ref => "&",
        }
    }
    pub fn from_token(token: &Token) -> Option<Self> {
        let Token::Symbol(symbol) = token else {
            return None;
        };
        Some(match symbol {
            Symbol::Ampersand => Self::Ref,
            Symbol::Bang => Self::Not,
            _ => {
                return None;
            }
        })
    }
}
