use super::{
    token::{FileSpan, TokenInstance},
    FilePos, Ident, Node,
};

#[derive(Debug, Clone)]
pub struct ParserError {
    pub msg: String,
    pub spans: Vec<FileSpan>,
}

pub struct ParserErrors {
    pub errs: Vec<ParserError>,
}

impl ParserError {
    pub fn from_instances(instances: &[&TokenInstance], msg: String) -> Self {
        ParserError {
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
        ParserError::from_instances(
            &[inst],
            format!("unexpected token {t:?}; expected {expected}"),
        )
    }
    pub fn write_for(&self, writer: &mut impl std::io::Write, file: &str) -> std::io::Result<()> {
        let after = if self.spans.is_empty() { "" } else { ":" };
        writeln!(writer, "error: {}{}", self.msg, after)?;
        for span in &self.spans {
            span.write_for(writer, file)?;
        }
        Ok(())
    }
}

impl ParserErrors {
    pub fn new() -> Self {
        Self { errs: Vec::new() }
    }
    pub fn add(&mut self, err: ParserError) {
        self.errs.push(err);
    }
}
