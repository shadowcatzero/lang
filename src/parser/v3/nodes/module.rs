use super::{
    AsmFunction, Function, Impl, Keyword, Node, Parsable, ParseResult, ParserMsg, ParserOutput,
    Struct, Symbol, Token, TokenCursor, Trait,
};
use std::fmt::Debug;

pub struct Module {
    pub traits: Vec<Node<Trait>>,
    pub structs: Vec<Node<Struct>>,
    pub functions: Vec<Node<Function>>,
    pub asm_fns: Vec<Node<AsmFunction>>,
    pub impls: Vec<Node<Impl>>,
}

impl Parsable for Module {
    fn parse(cursor: &mut TokenCursor, errors: &mut ParserOutput) -> ParseResult<Self> {
        let mut functions = Vec::new();
        let mut structs = Vec::new();
        let mut traits = Vec::new();
        let mut impls = Vec::new();
        let mut asm_fns = Vec::new();
        loop {
            let Some(next) = cursor.peek() else {
                break;
            };
            if let Token::Keyword(kw) = next.token {
                match kw {
                    Keyword::Fn => {
                        let res = Node::parse(cursor, errors);
                        functions.push(res.node);
                        if res.recover {
                            break;
                        }
                    }
                    Keyword::Struct => {
                        let res = Node::parse(cursor, errors);
                        structs.push(res.node);
                        if res.recover {
                            break;
                        }
                    }
                    Keyword::Trait => {
                        let res = Node::parse(cursor, errors);
                        traits.push(res.node);
                        if res.recover {
                            break;
                        }
                    }
                    Keyword::Impl => {
                        let res = Node::parse(cursor, errors);
                        impls.push(res.node);
                        if res.recover {
                            break;
                        }
                    }
                    Keyword::Asm => {
                        let res = Node::parse(cursor, errors);
                        asm_fns.push(res.node);
                        if res.recover {
                            break;
                        }
                    }
                    _ => {
                        errors.err(ParserMsg::unexpected_token(next, "a definition"));
                        cursor.next();
                    }
                }
            } else if next.is_symbol(Symbol::Semicolon) {
                errors.hint(ParserMsg::from_instances(
                    &[next],
                    "unneeded semicolon".to_string(),
                ));
                cursor.next();
            } else {
                errors.err(ParserMsg::unexpected_token(next, "a definition"));
                cursor.next();
            }
        }
        ParseResult::Ok(Self {
            functions,
            structs,
            traits,
            impls,
            asm_fns,
        })
    }
}

impl Debug for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for st in &self.structs {
            st.fmt(f)?;
            writeln!(f)?;
        }
        for t in &self.traits {
            t.fmt(f)?;
            writeln!(f)?;
        }
        for t in &self.impls {
            t.fmt(f)?;
            writeln!(f)?;
        }
        for func in &self.functions {
            func.fmt(f)?;
            writeln!(f)?;
        }
        for func in &self.asm_fns {
            func.fmt(f)?;
            writeln!(f)?;
        }
        Ok(())
    }
}
