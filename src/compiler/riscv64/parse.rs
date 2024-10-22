use super::{reg::*, AsmInstruction, Reg};
use crate::parser::{Parsable, ParseResult, ParserMsg, ParserOutput, Symbol, Token};

impl Parsable for AsmInstruction {
    fn parse(
        cursor: &mut crate::parser::TokenCursor,
        output: &mut ParserOutput,
    ) -> ParseResult<Self> {
        let t = cursor.expect_next()?;
        let span = t.span;
        match &t.token {
            Token::Word(w) => ParseResult::Ok(match w.as_str() {
                "ecall" => Self::Ecall,
                "li" => {
                    let dest = Reg::parse(cursor, output)?;
                    cursor.expect_sym(Symbol::Comma)?;
                    let imm = i64::parse(cursor, output)?;
                    Self::Li { dest, imm }
                }
                _ => {
                    return ParseResult::Err(ParserMsg::from_span(
                        span,
                        format!("Unknown instruction {}", w),
                    ))
                }
            }),
            _ => return ParseResult::Err(ParserMsg::unexpected_token(&t, "assembly or }")),
        }
    }
}

impl Parsable for Reg {
    fn parse(
        cursor: &mut crate::parser::TokenCursor,
        output: &mut ParserOutput,
    ) -> ParseResult<Self> {
        let next = cursor.expect_next()?;
        let Token::Word(word) = next.token else {
            return ParseResult::Err(ParserMsg::unexpected_token(&next, "a riscv register"));
        };
        ParseResult::Ok(match word.as_str() {
            "zero" => zero,
            "ra" => ra,
            "sp" => sp,
            "gp" => gp,
            "tp" => tp,
            "t0" => t0,
            "t1" => t1,
            "t2" => t2,
            "fp" => fp,
            "s0" => s0,
            "s1" => s1,
            "a0" => a0,
            "a1" => a1,
            "a2" => a2,
            "a3" => a3,
            "a4" => a4,
            "a5" => a5,
            "a6" => a6,
            "a7" => a7,
            "s2" => s2,
            "s3" => s3,
            "s4" => s4,
            "s5" => s5,
            "s6" => s6,
            "s7" => s7,
            "s8" => s8,
            "s9" => s9,
            "s10" => s10,
            "s11" => s11,
            "t3" => t3,
            "t4" => t4,
            "t5" => t5,
            "t6" => t6,
            other => {
                return ParseResult::Err(ParserMsg::from_span(
                    next.span,
                    format!("Unknown reg name {}", other),
                ));
            }
        })
    }
}

impl Parsable for i64 {
    fn parse(
        cursor: &mut crate::parser::TokenCursor,
        _output: &mut ParserOutput,
    ) -> ParseResult<Self> {
        let next = cursor.expect_next()?;
        let span = next.span;
        let Token::Word(word) = next.token else {
            return ParseResult::Err(ParserMsg::unexpected_token(&next, "an i32"));
        };
        let res = word.parse::<Self>();
        match res {
            Ok(int) => ParseResult::Ok(int),
            Err(_) => ParseResult::Err(ParserMsg::from_span(
                span,
                format!("Expected an i32, found {}", word),
            )),
        }
    }
}
