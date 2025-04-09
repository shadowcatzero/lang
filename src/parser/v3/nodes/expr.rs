use std::fmt::{Debug, Write};

use crate::common::FilePos;

use super::{
    op::{PInfixOp, UnaryOp},
    util::parse_list,
    CompilerMsg, Keyword, Node, NodeParsable, PAsmBlock, PBlock, PConstruct, PIdent, PLiteral,
    Parsable, ParseResult, ParserCtx, Symbol,
};

type BoxNode = Node<Box<PExpr>>;

pub enum PExpr {
    Lit(Node<PLiteral>),
    Ident(Node<PIdent>),
    BinaryOp(PInfixOp, BoxNode, BoxNode),
    UnaryOp(UnaryOp, BoxNode),
    Block(Node<PBlock>),
    Call(BoxNode, Vec<Node<PExpr>>),
    Group(BoxNode),
    AsmBlock(Node<PAsmBlock>),
    Construct(Node<PConstruct>),
    If(BoxNode, BoxNode),
    Loop(BoxNode),
    Break,
    Continue,
}

impl Parsable for PExpr {
    fn parse(ctx: &mut ParserCtx) -> ParseResult<Self> {
        let start = ctx.next_start();
        let next = ctx.expect_peek()?;
        let mut e1 = if next.is_symbol(Symbol::OpenParen) {
            ctx.next();
            if ctx.expect_peek()?.is_symbol(Symbol::CloseParen) {
                ctx.next();
                return ParseResult::Ok(PExpr::Lit(Node::new(
                    PLiteral::Unit,
                    ctx.next_start().char_span(),
                )));
            }
            let res = ctx.parse();
            if res.recover {
                ctx.seek_sym(Symbol::CloseParen);
            }
            ctx.expect_sym(Symbol::CloseParen)?;
            Self::Group(res.node.bx())
        } else if next.is_symbol(Symbol::OpenCurly) {
            Self::Block(PBlock::parse_node(ctx)?)
        } else if next.is_keyword(Keyword::If) {
            ctx.next();
            let cond = ctx.parse()?.bx();
            let body = ctx.parse()?.bx();
            Self::If(cond, body)
        } else if next.is_keyword(Keyword::Loop) {
            ctx.next();
            let body = ctx.parse()?.bx();
            Self::Loop(body)
        } else if next.is_keyword(Keyword::Break) {
            ctx.next();
            Self::Break
        } else if next.is_keyword(Keyword::Continue) {
            ctx.next();
            Self::Continue
        } else if next.is_keyword(Keyword::Asm) {
            ctx.next();
            Self::AsmBlock(ctx.parse()?)
        } else if let Some(op) = UnaryOp::from_token(next) {
            ctx.next();
            return ctx.parse().map(|n| {
                let n = n.bx();
                if let Some(box Self::BinaryOp(op2, n1, n2)) = n.inner {
                    let span = start.to(n1.span.end);
                    Self::BinaryOp(op2, Node::new(Self::UnaryOp(op, n1), span).bx(), n2)
                } else {
                    Self::UnaryOp(op, n)
                }
            });
        } else if let Some(val) = ctx.maybe_parse() {
            Self::Lit(val)
        } else {
            let res = ctx.parse();
            if res.node.is_some() {
                // TODO: this is extremely limiting
                // maybe parse generically and then during lowering figure out what's a function vs
                // struct vs etc like mentioned in main.rs
                if let Some(next) = ctx.peek()
                    && next.is_symbol(Symbol::OpenCurly)
                {
                    Self::Construct(ctx.parse_with(res.node)?)
                } else {
                    Self::Ident(res.node)
                }
            } else {
                let next = ctx.expect_peek()?;
                return ParseResult::Err(CompilerMsg::unexpected_token(next, "an expression"));
            }
        };
        let Some(mut next) = ctx.peek() else {
            return ParseResult::Ok(e1);
        };
        while next.is_symbol(Symbol::OpenParen) {
            ctx.next();
            let args = parse_list(ctx, Symbol::CloseParen)?;
            let end = ctx.prev_end();
            e1 = Self::Call(Node::new(Box::new(e1), start.to(end)), args);
            let Some(next2) = ctx.peek() else {
                return ParseResult::Ok(e1);
            };
            next = next2
        }
        let end = ctx.prev_end();
        let mut recover = false;
        let res = if let Some(op) = PInfixOp::from_token(&next.token) {
            ctx.next();
            let n1 = Node::new(e1, start.to(end)).bx();
            let res = ctx.parse();
            let n2 = res.node.bx();
            recover = res.recover;
            let (n1, op, n2) = fix_precedence(n1, op, n2, start);
            Self::BinaryOp(op, n1, n2)
        } else {
            e1
        };
        ParseResult::from_recover(res, recover)
    }
}

pub fn fix_precedence(
    mut n1: BoxNode,
    mut op: PInfixOp,
    mut n2: BoxNode,
    start: FilePos,
) -> (BoxNode, PInfixOp, BoxNode) {
    if let Some(box PExpr::BinaryOp(op2, _, _)) = n2.as_ref() {
        if op.precedence() >= op2.precedence() {
            let Some(box PExpr::BinaryOp(op2, n21, n22)) = n2.inner else {
                unreachable!();
            };
            let span = start.to(n21.span.end);
            let (n11, op1, n12) = fix_precedence(n1, op, n21, start);
            n1 = Node::new(PExpr::BinaryOp(op1, n11, n12), span).bx();
            op = op2;
            n2 = n22;
        }
    }
    (n1, op, n2)
}

impl Debug for PExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PExpr::Lit(c) => c.fmt(f)?,
            PExpr::Ident(n) => n.fmt(f)?,
            PExpr::Block(b) => b.fmt(f)?,
            PExpr::BinaryOp(op, e1, e2) => {
                write!(f, "({:?}", *e1)?;
                if op.pad() {
                    write!(f, " {} ", op.str())?;
                } else {
                    write!(f, "{}", op.str())?;
                }
                write!(f, "{:?})", *e2)?;
            }
            PExpr::Call(n, args) => {
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
            PExpr::UnaryOp(op, e) => write!(f, "({}{:?})", op.str(), e)?,
            PExpr::Group(inner) => inner.fmt(f)?,
            PExpr::AsmBlock(inner) => inner.fmt(f)?,
            PExpr::Construct(inner) => inner.fmt(f)?,
            PExpr::If(cond, res) => write!(f, "if {cond:?} then {res:?}")?,
            PExpr::Loop(res) => write!(f, "loop -> {res:?}")?,
            PExpr::Break => write!(f, "break")?,
            PExpr::Continue => write!(f, "continue")?,
        }
        Ok(())
    }
}
