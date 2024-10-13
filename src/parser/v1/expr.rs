use std::fmt::{Debug, Write};

use super::token::{Symbol, Token};
use super::{
    Body, Literal, MaybeResolved, Node, Operator, Parsable, ParserError, ParserErrors, Resolvable,
    Resolved, TokenCursor, Unresolved,
};

type BoxNode<R> = Node<Box<Expr<R>>, R>;

pub enum Expr<R: MaybeResolved> {
    Lit(Node<Literal, R>),
    Ident(String),
    BinaryOp(Operator, BoxNode<R>, BoxNode<R>),
    Block(Node<Body<R>, R>),
    Call(BoxNode<R>, Vec<Node<Expr<R>, R>>),
    Group(BoxNode<R>),
}

impl Expr<Unresolved> {
    pub fn ended_with_error(&self) -> bool {
        match self {
            Expr::Lit(_) => false,
            Expr::Ident(_) => false,
            Expr::BinaryOp(_, _, e) => e.is_err() || e.as_ref().is_ok_and(|e| e.ended_with_error()),
            Expr::Block(b) => b.is_err(),
            Expr::Call(_, _) => false,
            Expr::Group(_) => false,
        }
    }
}

impl Parsable for Expr<Unresolved> {
    fn parse(cursor: &mut TokenCursor, errors: &mut ParserErrors) -> Result<Self, ParserError> {
        let start = cursor.next_pos();
        let next = cursor.expect_peek()?;
        let mut e1 = if next.is_symbol(Symbol::OpenParen) {
            cursor.next();
            if cursor.expect_peek()?.is_symbol(Symbol::CloseParen) {
                cursor.next();
                return Ok(Expr::Lit(Node::new_unres(
                    Literal::Unit,
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
            Self::Lit(val)
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
            e1 = Self::Call(Node::new_unres(Box::new(e1), start.to(end)), vec![inner]);
            let Some(next2) = cursor.peek() else {
                return Ok(e1);
            };
            next = next2
        }
        let end = cursor.prev_end();
        Ok(if let Some(mut op) = Operator::from_token(&next.token) {
            cursor.next();
            let mut n1 = Node::new_unres(e1, start.to(end)).bx();
            let mut n2 = Node::parse(cursor, errors).bx();
            if let Ok(box Self::BinaryOp(op2, _, _)) = n2.as_ref() {
                if op.presedence() > op2.presedence() {
                    let Ok(box Self::BinaryOp(op2, n21, n22)) = n2.inner else {
                        unreachable!();
                    };
                    let end = n21.span.end;
                    n1 = Node::new_unres(Self::BinaryOp(op, n1, n21), start.to(end)).bx();
                    op = op2;
                    n2 = n22;
                }
            }
            Self::BinaryOp(op, n1, n2)
        } else {
            e1
        })
    }
}

impl Resolvable<Expr<Resolved>> for Expr<Unresolved> {
    fn resolve(self) -> Result<Expr<Resolved>, ()> {
        Ok(match self {
            Expr::Lit(l) => Expr::Lit(l.resolve()?),
            Expr::Ident(n) => Expr::Ident(n),
            Expr::BinaryOp(o, e1, e2) => Expr::BinaryOp(o, e1.resolve()?, e2.resolve()?),
            Expr::Block(b) => Expr::Block(b.resolve()?),
            Expr::Call(f, args) => Expr::Call(
                f.resolve()?,
                args.into_iter()
                    .map(|arg| arg.resolve())
                    .collect::<Result<_, ()>>()?,
            ),
            Expr::Group(e) => Expr::Group(e.resolve()?),
        })
    }
}

impl Debug for Expr<Unresolved> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Lit(c) => c.fmt(f)?,
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
