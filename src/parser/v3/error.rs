use crate::ir::{FilePos, FileSpan};

use super::{token::TokenInstance, Ident, Node};

#[derive(Debug, Clone)]
pub struct ParserMsg {
    pub msg: String,
    pub spans: Vec<FileSpan>,
}

pub struct ParserOutput {
    pub errs: Vec<ParserMsg>,
    pub hints: Vec<ParserMsg>,
}

impl ParserMsg {
    pub fn from_instances(instances: &[&TokenInstance], msg: String) -> Self {
        ParserMsg {
            msg,
            spans: instances.iter().map(|i| i.span).collect(),
        }
    }
    pub fn from_msg(msg: String) -> Self {
        Self {
            msg,
            spans: Vec::new(),
        }
    }
    pub fn from_span(span: FileSpan, msg: String) -> Self {
        Self {
            msg,
            spans: vec![span],
        }
    }
    pub fn identifier_not_found(id: &Node<Ident>) -> Self {
        Self {
            msg: format!("Identifier '{}' not found", id.as_ref().unwrap().val()),
            spans: vec![id.span],
        }
    }
    pub fn at(pos: FilePos, msg: String) -> Self {
        Self {
            msg,
            spans: vec![FileSpan::at(pos)],
        }
    }
    pub fn unexpected_end() -> Self {
        Self::from_msg("unexpected end of input".to_string())
    }
    pub fn unexpected_token(inst: &TokenInstance, expected: &str) -> Self {
        let t = &inst.token;
        ParserMsg::from_instances(
            &[inst],
            format!("unexpected token {t:?}; expected {expected}"),
        )
    }
    pub fn write_for(&self, ty: &str, writer: &mut impl std::io::Write, file: &str) -> std::io::Result<()> {
        let after = if self.spans.is_empty() { "" } else { ":" };
        writeln!(writer, "{}: {}{}", ty, self.msg, after)?;
        for span in &self.spans {
            span.write_for(writer, file)?;
        }
        Ok(())
    }
}

impl ParserOutput {
    pub fn new() -> Self {
        Self {
            errs: Vec::new(),
            hints: Vec::new(),
        }
    }
    pub fn err(&mut self, err: ParserMsg) {
        self.errs.push(err);
    }
    pub fn hint(&mut self, err: ParserMsg) {
        self.hints.push(err);
    }
    pub fn write_for(&self, out: &mut impl std::io::Write, file: &str) {
        for err in &self.errs {
            err.write_for("error", out, file).unwrap();
        }
        for hint in &self.hints {
            hint.write_for("hint", out, file).unwrap();
        }
    }
}
