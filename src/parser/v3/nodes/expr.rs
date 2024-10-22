use std::fmt::{Debug, Write};

use super::{
    op::{BinaryOp, UnaryOp},
    util::parse_list,
    AsmBlock, Block, Ident, Keyword, Literal, Node, NodeParsable, Parsable, ParseResult, ParserMsg,
    ParserOutput, Symbol, TokenCursor,
};

type BoxNode = Node<Box<Expr>>;

pub enum Expr {
    Lit(Node<Literal>),
    Ident(Node<Ident>),
    BinaryOp(BinaryOp, BoxNode, BoxNode),
    UnaryOp(UnaryOp, BoxNode),
    Block(Node<Block>),
    Call(BoxNode, Vec<Node<Expr>>),
    Group(BoxNode),
    AsmBlock(Node<AsmBlock>),
}

impl Parsable for Expr {
    fn parse(cursor: &mut TokenCursor, output: &mut ParserOutput) -> ParseResult<Self> {
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
            let res = Node::parse(cursor, output);
            if res.recover {
                cursor.seek_sym(Symbol::CloseParen);
            }
            cursor.expect_sym(Symbol::CloseParen)?;
            Self::Group(res.node.bx())
        } else if next.is_symbol(Symbol::OpenCurly) {
            Self::Block(Block::parse_node(cursor, output)?)
        } else if next.is_keyword(Keyword::Asm) {
            cursor.next();
            Self::AsmBlock(Node::parse(cursor, output)?)
        } else if let Some(op) = UnaryOp::from_token(next) {
            cursor.next();
            return Node::parse(cursor, output).map(|n| {
                let n = n.bx();
                if let Some(box Self::BinaryOp(op2, n1, n2)) = n.inner {
                    let span = start.to(n1.span.end);
                    Self::BinaryOp(op2, Node::new(Self::UnaryOp(op, n1), span).bx(), n2)
                } else {
                    Self::UnaryOp(op, n)
                }
            });
        } else if let Some(val) = Node::maybe_parse(cursor, output) {
            Self::Lit(val)
        } else {
            let res = Node::parse(cursor, &mut ParserOutput::new());
            if res.node.is_some() {
                Self::Ident(res.node)
            } else {
                let next = cursor.expect_peek()?;
                return ParseResult::Err(ParserMsg::unexpected_token(next, "an expression"));
            }
        };
        let Some(mut next) = cursor.peek() else {
            return ParseResult::Ok(e1);
        };
        while next.is_symbol(Symbol::OpenParen) {
            cursor.next();
            let args = parse_list(cursor, output, Symbol::CloseParen)?;
            let end = cursor.prev_end();
            e1 = Self::Call(Node::new(Box::new(e1), start.to(end)), args);
            let Some(next2) = cursor.peek() else {
                return ParseResult::Ok(e1);
            };
            next = next2
        }
        let end = cursor.prev_end();
        let mut recover = false;
        let res = if let Some(mut op) = BinaryOp::from_token(&next.token) {
            cursor.next();
            let mut n1 = Node::new(e1, start.to(end)).bx();
            let res = Node::parse(cursor, output);
            let mut n2 = res.node.bx();
            recover = res.recover;
            if let Some(box Self::BinaryOp(op2, _, _)) = n2.as_ref() {
                if op.presedence() > op2.presedence() {
                    let Some(box Self::BinaryOp(op2, n21, n22)) = n2.inner else {
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

impl Debug for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Lit(c) => c.fmt(f)?,
            Expr::Ident(n) => n.fmt(f)?,
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
            Expr::AsmBlock(inner) => inner.fmt(f)?,
        }
        Ok(())
    }
}
