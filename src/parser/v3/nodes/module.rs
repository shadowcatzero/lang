use super::{
    PFunction, PImpl, Keyword, Node, Parsable, ParseResult, ParserCtx, CompilerMsg,
    PStruct, Symbol, Token, PTrait,
};
use std::fmt::Debug;

pub struct PModule {
    pub traits: Vec<Node<PTrait>>,
    pub structs: Vec<Node<PStruct>>,
    pub functions: Vec<Node<PFunction>>,
    pub impls: Vec<Node<PImpl>>,
}

impl Parsable for PModule {
    fn parse(ctx: &mut ParserCtx) -> ParseResult<Self> {
        let mut functions = Vec::new();
        let mut structs = Vec::new();
        let mut traits = Vec::new();
        let mut impls = Vec::new();
        loop {
            let Some(next) = ctx.peek() else {
                break;
            };
            if let Token::Keyword(kw) = next.token {
                match kw {
                    Keyword::Fn => {
                        let res = ctx.parse();
                        functions.push(res.node);
                        if res.recover {
                            break;
                        }
                    }
                    Keyword::Struct => {
                        let res = ctx.parse();
                        structs.push(res.node);
                        if res.recover {
                            break;
                        }
                    }
                    Keyword::Trait => {
                        let res = ctx.parse();
                        traits.push(res.node);
                        if res.recover {
                            break;
                        }
                    }
                    Keyword::Impl => {
                        let res = ctx.parse();
                        impls.push(res.node);
                        if res.recover {
                            break;
                        }
                    }
                    _ => {
                        ctx.err(CompilerMsg::unexpected_token(next, "a definition"));
                        ctx.next();
                    }
                }
            } else if next.is_symbol(Symbol::Semicolon) {
                ctx.hint(CompilerMsg::from_instances(
                    &[next],
                    "unneeded semicolon".to_string(),
                ));
                ctx.next();
            } else {
                ctx.err(CompilerMsg::unexpected_token(next, "a definition"));
                ctx.next();
            }
        }
        ParseResult::Ok(Self {
            functions,
            structs,
            traits,
            impls,
        })
    }
}

impl Debug for PModule {
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
        Ok(())
    }
}
