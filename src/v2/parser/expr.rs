use super::{util::WHITESPACE_SET, Body, CharCursor, ParserError};
use std::{collections::HashSet, fmt::Debug, sync::LazyLock};

static SYMBOLS: LazyLock<HashSet<char>> = LazyLock::new(|| {
    let mut set = HashSet::new();
    for o in Operator::ALL {
        for c in o.str().chars() {
            set.insert(c);
        }
    }
    set
});

static IDENT_END: LazyLock<HashSet<char>> = LazyLock::new(|| {
    let mut set = WHITESPACE_SET.clone();
    let symbols = &SYMBOLS;
    set.extend(symbols.iter().chain(&[';', '(', ')']));
    set
});

#[derive(Debug)]
pub enum Val {
    String(String),
    Number(String),
    Unit,
}

pub enum Expr {
    Block(Body),
    Val(Val),
    Ident(String),
    BinaryOp(Operator, Box<Expr>, Box<Expr>),
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

impl Expr {
    pub fn parse(cursor: &mut CharCursor) -> Result<Self, ParserError> {
        cursor.skip_whitespace();
        let Some(next) = cursor.peek() else {
            return Ok(Self::Val(Val::Unit));
        };
        let mut e1 = match next {
            '(' => {
                cursor.advance();
                let expr = Self::parse(cursor)?;
                cursor.skip_whitespace();
                cursor.expect_char(')')?;
                expr
            }
            '{' => {
                Self::Block(Body::parse(cursor)?)
            }
            _ => {
                if let Some(val) = Val::parse_nonunit(cursor)? {
                    Self::Val(val)
                } else {
                    let name = cursor.until(&IDENT_END);
                    Self::Ident(name)
                }
            }
        };
        cursor.skip_whitespace();
        let Some(mut next) = cursor.peek() else {
            return Ok(e1);
        };
        while next == '(' {
            cursor.advance();
            let inner = Self::parse(cursor)?;
            cursor.skip_whitespace();
            cursor.expect_char(')')?;
            e1 = Self::Call(Box::new(e1), vec![inner]);
            let Some(next2) = cursor.peek() else {
                return Ok(e1);
            };
            next = next2
        }
        if let Some(op) = Operator::parse(cursor) {
            let e2 = Self::parse(cursor)?;
            return Ok(if let Self::BinaryOp(op_next, e2, e3) = e2 {
                if op.presedence() > op_next.presedence() {
                    Self::BinaryOp(op_next, Box::new(Self::BinaryOp(op, Box::new(e1), e2)), e3)
                } else {
                    Self::BinaryOp(op, Box::new(e1), Box::new(Self::BinaryOp(op_next, e2, e3)))
                }
            } else {
                Self::BinaryOp(op, Box::new(e1), Box::new(e2))
            });
        };
        Ok(e1)
    }
}

impl Val {
    pub fn parse_nonunit(cursor: &mut CharCursor) -> Result<Option<Self>, ParserError> {
        let Some(next) = cursor.peek() else {
            return Ok(None);
        };
        Ok(Some(match next {
            '"' => {
                cursor.advance();
                let mut str = String::new();
                loop {
                    let mut next = cursor.expect_next()?;
                    if next == '"' {
                        break;
                    }
                    if next == '\\' {
                        next = match cursor.expect_next()? {
                            '"' => '"',
                            c => {
                                return Err(ParserError::at(
                                    cursor.pos(),
                                    format!("unexpected escape char '{c}'"),
                                ))
                            }
                        }
                    }
                    str.push(next);
                }
                Self::String(str)
            }
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                let mut str = String::new();
                loop {
                    let Some(next) = cursor.peek() else {
                        break;
                    };
                    match next {
                        '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                            str.push(next);
                        }
                        _ => break,
                    }
                    cursor.advance();
                }
                Self::Number(str)
            }
            _ => {
                return Ok(None);
            }
        }))
    }
}

impl Operator {
    const ALL: [Self; 7] = [
        Self::Add,
        Self::Sub,
        Self::Mul,
        Self::Div,
        Self::Offset,
        Self::GreaterThan,
        Self::LessThan,
    ];
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
    pub fn parse(cursor: &mut CharCursor) -> Option<Self> {
        let res = match cursor.peek()? {
            '+' => Operator::Add,
            '-' => Operator::Sub,
            '*' => Operator::Mul,
            '/' => Operator::Div,
            '.' => Operator::Offset,
            _ => return None,
        };
        for _ in 0..res.str().len() {
            cursor.advance();
        }
        Some(res)
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
            Expr::Block(b) => write!(f, "{:?}", b)?,
            Expr::Ident(n) => f.write_str(n)?,
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
                write!(f, "(")?;
                if let Some(a) = args.first() {
                    a.fmt(f)?;
                }
                for arg in args.iter().skip(1) {
                    write!(f, ", ")?;
                    arg.fmt(f)?;
                }
                write!(f, ")")?;
            }
            Expr::Val(v) => {
                write!(f, "{:?}", v)?;
            }
        }
        Ok(())
    }
}
