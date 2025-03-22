use super::{FilePos, FileSpan};

#[derive(Debug, Clone)]
pub struct CompilerMsg {
    pub msg: String,
    pub spans: Vec<FileSpan>,
}

pub struct CompilerOutput {
    pub errs: Vec<CompilerMsg>,
    pub hints: Vec<CompilerMsg>,
}

impl CompilerMsg {
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
    pub fn at(pos: FilePos, msg: String) -> Self {
        Self {
            msg,
            spans: vec![FileSpan::at(pos)],
        }
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

impl CompilerOutput {
    pub fn new() -> Self {
        Self {
            errs: Vec::new(),
            hints: Vec::new(),
        }
    }
    pub fn err(&mut self, msg: CompilerMsg) {
        self.errs.push(msg);
    }
    pub fn hint(&mut self, msg: CompilerMsg) {
        self.hints.push(msg);
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
