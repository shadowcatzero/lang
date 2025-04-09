use super::{Symbol, Token};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PInfixOp {
    Add,
    Sub,
    Mul,
    Div,
    LessThan,
    GreaterThan,
    Access,
    Assign,
}

impl PInfixOp {
    pub fn precedence(&self) -> u32 {
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
    pub fn traitt(&self) -> &str {
        match self {
            Self::Add => "Add",
            Self::Sub => "Sub",
            Self::Mul => "Mul",
            Self::Div => "Div",
            Self::LessThan => "LessThan",
            Self::GreaterThan => "GreaterThan",
            Self::Access => "Access",
            Self::Assign => "Assign",
        }
    }
}

pub enum UnaryOp {
    Not,
    Ref,
    Deref,
}

impl PInfixOp {
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
}

impl UnaryOp {
    pub fn str(&self) -> &str {
        match self {
            Self::Not => "!",
            Self::Ref => "&",
            Self::Deref => "*",
        }
    }
    pub fn from_token(token: &Token) -> Option<Self> {
        let Token::Symbol(symbol) = token else {
            return None;
        };
        Some(match symbol {
            Symbol::Ampersand => Self::Ref,
            Symbol::Bang => Self::Not,
            Symbol::Asterisk => Self::Deref,
            _ => {
                return None;
            }
        })
    }
}
