#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Keyword {
    Fn,
    Let,
    If,
    Return,
}

impl Keyword {
    pub fn from_string(str: &str) -> Option<Self> {
        Some(match str {
            "fn" => Self::Fn,
            "let" => Self::Let,
            "if" => Self::If,
            "return" => Self::Return,
            _ => return None,
        })
    }
}
