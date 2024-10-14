use std::fmt::{Debug, Write};

use super::token::{Symbol, Token};
use super::{
    BinaryOperator, Body, Literal, MaybeResolved, Node, NodeParsable, Parsable, ParseResult,
    ParserError, ParserErrors, Resolvable, Resolved, TokenCursor, UnaryOperator, Unresolved,
};

type BoxNode<R> = Node<Box<Expr<R>>, R>;

pub enum Expr<R: MaybeResolved> {
    Lit(Node<Literal, R>),
    Ident(String),
    BinaryOp(BinaryOperator, BoxNode<R>, BoxNode<R>),
    UnaryOp(UnaryOperator, BoxNode<R>),
    Block(Node<Body<R>, R>),
    Call(BoxNode<R>, Vec<Node<Expr<R>, R>>),
    Group(BoxNode<R>),
}

impl Parsable for Expr<Unresolved> {
    fn parse(cursor: &mut TokenCursor, errors: &mut ParserErrors) -> ParseResult<Self> {
        let start = cursor.next_pos();
        let next = cursor.expect_peek()?;
        let mut e1 = if next.is_symbol(Symbol::OpenParen) {
            cursor.next();
            if cursor.expect_peek()?.is_symbol(Symbol::CloseParen) {
                cursor.next();
                return ParseResult::Ok(Expr::Lit(Node::new(
                    Literal::Unit,
                    cursor.next_pos().char_span(),
                )));
            }
            let res = Node::parse(cursor, errors);
            if res.recover {
                cursor.seek_sym(Symbol::CloseParen);
            }
            cursor.expect_sym(Symbol::CloseParen)?;
            Self::Group(res.node.bx())
        } else if next.is_symbol(Symbol::OpenCurly) {
            Self::Block(Body::parse_node(cursor, errors)?)
        } else if let Some(op) = UnaryOperator::from_token(next) {
            cursor.next();
            return Node::parse(cursor, errors).map(|n| {
                let n = n.bx();
                if let Ok(box Self::BinaryOp(op2, n1, n2)) = n.inner {
                    let span = start.to(n1.span.end);
                    Self::BinaryOp(op2, Node::new(Self::UnaryOp(op, n1), span).bx(), n2)
                } else {
                    Self::UnaryOp(op, n)
                }
            });
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
                    return ParseResult::Err(ParserError::unexpected_token(next, "an expression"));
                }
            }
        };
        let Some(mut next) = cursor.peek() else {
            return ParseResult::Ok(e1);
        };
        while next.is_symbol(Symbol::OpenParen) {
            cursor.next();
            let mut args = Vec::new();
            while !cursor.expect_peek()?.is_symbol(Symbol::CloseParen) {
                let res = Node::<Expr<Unresolved>, Unresolved>::parse(cursor, errors);
                args.push(res.node);
                if res.recover {
                    cursor.seek_sym(Symbol::CloseParen);
                    break;
                }
            }
            cursor.expect_sym(Symbol::CloseParen)?;
            let end = cursor.prev_end();
            e1 = Self::Call(Node::new(Box::new(e1), start.to(end)), args);
            let Some(next2) = cursor.peek() else {
                return ParseResult::Ok(e1);
            };
            next = next2
        }
        let end = cursor.prev_end();
        let mut recover = false;
        let res = if let Some(mut op) = BinaryOperator::from_token(&next.token) {
            cursor.next();
            let mut n1 = Node::new(e1, start.to(end)).bx();
            let res = Node::parse(cursor, errors);
            let mut n2 = res.node.bx();
            recover = res.recover;
            if let Ok(box Self::BinaryOp(op2, _, _)) = n2.as_ref() {
                if op.presedence() > op2.presedence() {
                    let Ok(box Self::BinaryOp(op2, n21, n22)) = n2.inner else {
                        unreachable!();
                    };
                    let end = n21.span.end;
                    n1 = Node::new(Self::BinaryOp(op, n1, n21), start.to(end)).bx();
                    op = op2;
                    n2 = n22;
                }
            }
            Self::BinaryOp(op, n1, n2)
        } else {
            e1
        };
        ParseResult::from_recover(res, recover)
    }
}

impl Resolvable<Expr<Resolved>> for Expr<Unresolved> {
    fn resolve(self) -> Result<Expr<Resolved>, ()> {
        Ok(match self {
            Expr::Lit(l) => Expr::Lit(l.resolve()?),
            Expr::Ident(n) => Expr::Ident(n),
            Expr::BinaryOp(o, e1, e2) => Expr::BinaryOp(o, e1.resolve()?, e2.resolve()?),
            Expr::UnaryOp(o, e) => Expr::UnaryOp(o, e.resolve()?),
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
            Expr::UnaryOp(op, e) => {
                write!(f, "(")?;
                write!(f, "{}", op.str())?;
                write!(f, "{:?})", *e)?;
            }
            Expr::Group(inner) => inner.fmt(f)?,
        }
        Ok(())
    }
}
