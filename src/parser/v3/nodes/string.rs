use crate::{
    common::CompilerMsg,
    parser::{Parsable, ParseResult},
};

pub struct PString(pub String);

impl Parsable for PString {
    fn parse(ctx: &mut crate::parser::ParserCtx) -> ParseResult<Self> {
        let cursor = ctx.cursor.chars();
        let mut str = String::new();
        loop {
            let c = cursor.expect_next()?;
            if c == '"' {
                return ParseResult::Ok(Self(str));
            }
            str.push(match c {
                '\\' => {
                    let start = cursor.prev_pos();
                    let next = cursor.expect_next()?;
                    match next {
                        '"' => '"',
                        '\'' => '\'',
                        't' => '\t',
                        'n' => '\n',
                        '0' => '\0',
                        other => {
                            let end = cursor.prev_pos();
                            ctx.output.err(CompilerMsg {
                                msg: format!("Unknown escape sequence '\\{}'", other),
                                spans: vec![start.to(end)],
                            });
                            other
                        }
                    }
                }
                _ => c,
            })
        }
    }
}
