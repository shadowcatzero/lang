use crate::compiler::riscv64::AsmInstruction;

use super::{Parsable, ParseResult, Symbol};

#[derive(Debug)]
pub struct AsmBlock {
    pub instructions: Vec<AsmInstruction>,
}

impl Parsable for AsmBlock {
    fn parse(
        cursor: &mut super::TokenCursor,
        output: &mut super::ParserOutput,
    ) -> ParseResult<Self> {
        cursor.expect_sym(Symbol::OpenCurly)?;
        let mut instructions = Vec::new();
        while !cursor.expect_peek()?.is_symbol(Symbol::CloseCurly) {
            instructions.push(AsmInstruction::parse(cursor, output)?);
        }
        cursor.expect_sym(Symbol::CloseCurly)?;
        ParseResult::Ok(Self { instructions })
    }
}
