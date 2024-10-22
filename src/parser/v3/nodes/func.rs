use super::{
    util::parse_list, Block, Ident, Keyword, Node, Parsable, ParseResult, ParserOutput, SelfVar,
    Symbol, TokenCursor, Type, VarDef,
};
use std::fmt::Debug;

pub struct FunctionHeader {
    pub name: Node<Ident>,
    pub sel: Option<Node<SelfVar>>,
    pub args: Vec<Node<VarDef>>,
    pub ret: Option<Node<Type>>,
}

pub struct Function {
    pub header: Node<FunctionHeader>,
    pub body: Node<Block>,
}

impl Parsable for FunctionHeader {
    fn parse(cursor: &mut TokenCursor, output: &mut ParserOutput) -> ParseResult<Self> {
        cursor.expect_kw(Keyword::Fn)?;
        let name = Node::parse(cursor, output)?;
        cursor.expect_sym(Symbol::OpenParen)?;
        let sel = Node::maybe_parse(cursor, output);
        if sel.is_some() {
            if let Err(err) = cursor.expect_sym(Symbol::Comma) {
                output.err(err);
                cursor.seek_syms(&[Symbol::Comma, Symbol::CloseParen]);
                if cursor.peek().is_some_and(|i| i.is_symbol(Symbol::Comma)) {
                    cursor.next();
                }
            }
        }
        let args = parse_list(cursor, output, Symbol::CloseParen)?;
        let ret = if cursor.peek().is_some_and(|i| i.is_symbol(Symbol::Arrow)) {
            cursor.next();
            Some(Node::parse(cursor, output)?)
        } else {
            None
        };
        ParseResult::Ok(Self {
            name,
            args,
            sel,
            ret,
        })
    }
}

impl Parsable for Function {
    fn parse(cursor: &mut TokenCursor, output: &mut ParserOutput) -> ParseResult<Self> {
        let header = Node::parse(cursor, output)?;
        let body = Node::parse(cursor, output)?;
        ParseResult::Ok(Self { header, body })
    }
}

impl Debug for FunctionHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("fn ")?;
        self.name.fmt(f)?;
        f.write_str("(")?;
        if let Some(s) = &self.sel {
            s.fmt(f)?;
            if self.args.first().is_some() {
                f.write_str(", ")?;
            }
        }
        if let Some(a) = self.args.first() {
            a.fmt(f)?;
        }
        for arg in self.args.iter().skip(1) {
            f.write_str(", ")?;
            arg.fmt(f)?;
        }
        f.write_str(")")?;
        if let Some(ret) = &self.ret {
            write!(f, " -> {:?}", ret)?;
        }
        Ok(())
    }
}
impl Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.header.fmt(f)?;
        f.write_str(" ")?;
        self.body.fmt(f)?;
        Ok(())
    }
}
