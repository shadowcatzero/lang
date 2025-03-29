#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Keyword {
    Fn,
    Let,
    If,
    Return,
    Break,
    Loop,
    Struct,
    Trait,
    Impl,
    For,
    Asm,
    Funne,
}

impl Keyword {
    pub fn from_string(str: &str) -> Option<Self> {
        Some(match str {
            "fn" => Self::Fn,
            "struct" => Self::Struct,
            "let" => Self::Let,
            "if" => Self::If,
            "for" => Self::For,
            "return" => Self::Return,
            "break" => Self::Break,
            "loop" => Self::Loop,
            "trait" => Self::Trait,
            "impl" => Self::Impl,
            "asm" => Self::Asm,
            "funne" => Self::Funne,
            _ => return None,
        })
    }
}
