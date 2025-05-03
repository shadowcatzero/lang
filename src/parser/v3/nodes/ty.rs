use std::fmt::Debug;

use super::{util::parse_list, Node, PIdent, Parsable, ParseResult, ParserCtx, Symbol};

type BoxNode = Node<Box<PType>>;

pub enum PType {
    Member(BoxNode, Node<PIdent>),
    Ref(BoxNode),
    Generic(BoxNode, Vec<Node<PType>>),
    Ident(PIdent),
}

pub struct PGenericDef {
    pub name: Node<PIdent>,
}

impl Parsable for PType {
    fn parse(ctx: &mut ParserCtx) -> ParseResult<Self> {
        let start = ctx.next_start();
        let mut cur = ctx.parse()?.map(PType::Ident);
        loop {
            let span = start.to(ctx.prev_end());
            let Some(next) = ctx.peek() else {
                break;
            };
            if next.is_symbol(Symbol::Ampersand) {
                ctx.next();
                cur = Node::new(PType::Ref(cur.bx()), span);
                continue;
            } else if next.is_symbol(Symbol::OpenAngle) {
                ctx.next();
                let args = parse_list(ctx, Symbol::CloseAngle)?;
                cur = Node::new(PType::Generic(cur.bx(), args), span);
                continue;
            } else if next.is_symbol(Symbol::DoubleColon) {
                ctx.next();
                let mem = ctx.parse()?;
                cur = Node::new(PType::Member(cur.bx(), mem), span);
            }
            break;
        }
        ParseResult::Node(cur)
    }
}

impl Parsable for PGenericDef {
    fn parse(ctx: &mut ParserCtx) -> ParseResult<Self> {
        ParseResult::Ok(Self { name: ctx.parse()? })
    }
}

impl Debug for PType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PType::Member(node, name) => write!(f, "{:?}.{:?}", node, name)?,
            PType::Ref(node) => write!(f, "{:?}&", node)?,
            PType::Generic(node, args) => write!(f, "{:?}<{:?}>", node, args)?,
            PType::Ident(node) => node.fmt(f)?,
        }
        Ok(())
    }
}

impl Debug for PGenericDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.name)?;
        Ok(())
    }
}
