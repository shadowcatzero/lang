use std::fmt::{Debug, Write};

use crate::{common::FilePos, ir::MemberTy, parser::NodeParsableWith};

use super::{
    op::{InfixOp, PostfixOp},
    util::parse_list,
    CompilerMsg, Keyword, Node, PAsmBlock, PBlock, PIdent, PLiteral, PMap, PType, Parsable,
    ParseResult, ParserCtx, Symbol,
};

type BoxNode = Node<Box<PExpr>>;

pub enum PExpr {
    Lit(PLiteral),
    Ident(Node<PIdent>),
    BinaryOp(InfixOp, BoxNode, BoxNode),
    PostfixOp(BoxNode, PostfixOp),
    Block(Node<PBlock>),
    Call(BoxNode, Vec<Node<PExpr>>),
    Group(BoxNode),
    Member(BoxNode, MemberTy, Node<PIdent>),
    Generic(BoxNode, Vec<Node<PType>>),
    AsmBlock(Node<PAsmBlock>),
    Construct(BoxNode, Node<PMap>),
    If(BoxNode, BoxNode),
    Loop(BoxNode),
    Break,
    Continue,
}

impl Parsable for PExpr {
    fn parse(ctx: &mut ParserCtx) -> ParseResult<Self> {
        let start = ctx.next_start();
        let mut e1 = Self::parse_unit_postfix(ctx)?;
        while let Some(op) = ctx
            .peek()
            .map(|next| InfixOp::from_token(&next.token))
            .flatten()
        {
            let span = start.to(ctx.prev_end());
            ctx.next();
            let n2 = ctx.parse()?.bx();
            let (n1, op, n2) = fix_precedence(Node::new(e1, span).bx(), op, n2, start);
            e1 = Self::BinaryOp(op, n1, n2);
        }
        return ParseResult::Ok(e1);
    }
}

impl PExpr {
    fn parse_unit_postfix(ctx: &mut ParserCtx) -> ParseResult<Self> {
        let start = ctx.next_start();
        // first get unit
        let mut e1 = Self::parse_unit(ctx)?;
        // then apply post ops
        loop {
            let span = start.to(ctx.prev_end());
            let Some(next) = ctx.peek() else {
                break;
            };
            if next.is_symbol(Symbol::OpenParen) {
                ctx.next();
                let args = parse_list(ctx, Symbol::CloseParen)?;
                e1 = Self::Call(Node::new(e1, span).bx(), args);
                continue;
            } else if next.is_symbol(Symbol::OpenCurly) {
                ctx.next();
                let map = ctx.parse()?;
                e1 = Self::Construct(Node::new(e1, span).bx(), map);
                continue;
            } else if next.is_symbol(Symbol::Dot) {
                ctx.next();
                let field = ctx.parse()?;
                e1 = Self::Member(Node::new(e1, span).bx(), MemberTy::Field, field);
                continue;
            } else if next.is_symbol(Symbol::DoubleColon) {
                ctx.next();
                if ctx.peek().is_some_and(|i| i.is_symbol(Symbol::OpenAngle)) {
                    ctx.next();
                    let gargs = parse_list(ctx, Symbol::CloseAngle)?;
                    e1 = Self::Generic(Node::new(e1, span).bx(), gargs);
                } else {
                    let field = ctx.parse()?;
                    e1 = Self::Member(Node::new(e1, span).bx(), MemberTy::Member, field);
                }
                continue;
            } else if let Some(op) = PostfixOp::from_token(next) {
                ctx.next();
                e1 = Self::PostfixOp(Node::new(e1, span).bx(), op);
                continue;
            }
            break;
        }
        return ParseResult::Ok(e1);
    }
    fn parse_unit(ctx: &mut ParserCtx) -> ParseResult<Self> {
        let next = ctx.expect_peek()?;
        return ParseResult::Ok(if next.is_symbol(Symbol::OpenParen) {
            ctx.next();
            if ctx.expect_peek()?.is_symbol(Symbol::CloseParen) {
                ctx.next();
                return ParseResult::Ok(PExpr::Lit(PLiteral::Unit));
            }
            let res = ctx.parse();
            if res.recover {
                ctx.seek_sym(Symbol::CloseParen);
            }
            ctx.expect_sym(Symbol::CloseParen)?;
            Self::Group(res.node.bx())
        } else if next.is_symbol(Symbol::OpenCurly) {
            ctx.next();
            Self::Block(PBlock::parse_node(ctx, Some(Symbol::CloseCurly))?)
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
        } else if let Some(res) = ctx.maybe_parse::<PLiteral>() {
            return ParseResult::Wrap(res.map(Self::Lit));
        } else {
            let res = ctx.parse();
            if res.node.is_some() {
                Self::Ident(res.node)
            } else {
                let next = ctx.expect_peek()?;
                return ParseResult::Err(CompilerMsg::unexpected_token(next, "an expression"));
            }
        });
    }
}

pub fn fix_precedence(
    mut n1: BoxNode,
    mut op: InfixOp,
    mut n2: BoxNode,
    start: FilePos,
) -> (BoxNode, InfixOp, BoxNode) {
    if let Some(box PExpr::BinaryOp(op2, _, _)) = n2.as_ref() {
        if op.precedence() > op2.precedence() {
            let Some(box PExpr::BinaryOp(op2, n21, n22)) = n2.inner else {
                unreachable!();
            };
            let span = start.to(n21.origin.end);
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
            PExpr::PostfixOp(e, op) => write!(f, "({:?}{})", e, op.str())?,
            PExpr::Group(inner) => inner.fmt(f)?,
            PExpr::AsmBlock(inner) => inner.fmt(f)?,
            PExpr::Construct(node, inner) => write!(f, "{:?}{:?}", node, inner)?,
            PExpr::If(cond, res) => write!(f, "if {cond:?} then {res:?}")?,
            PExpr::Loop(res) => write!(f, "loop -> {res:?}")?,
            PExpr::Break => write!(f, "break")?,
            PExpr::Continue => write!(f, "continue")?,
            PExpr::Member(e1, ty, name) => write!(f, "{:?}{}{:?}", e1, ty.sep(), name)?,
            PExpr::Generic(e1, gargs) => write!(f, "{:?}<{:?}>", e1, gargs)?,
        }
        Ok(())
    }
}
