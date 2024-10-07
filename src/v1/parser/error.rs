use super::{
    token::{FileSpan, TokenInstance},
    FilePos,
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
            format!("Unexpected token {t:?}; expected {expected}"),
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
    pub fn add(&mut self, err: ParserError) {
        self.errs.push(err);
    }
}
