use super::{FileMap, FilePos, FileSpan};

#[derive(Debug, Clone)]
pub struct CompilerMsg {
    pub msg: String,
    pub spans: Vec<FileSpan>,
}

pub struct CompilerOutput {
    pub file_map: FileMap,
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
    pub fn new(msg: String, span: FileSpan) -> Self {
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
    pub fn write_to(
        &self,
        ty: &str,
        writer: &mut impl std::io::Write,
        map: &FileMap,
    ) -> std::io::Result<()> {
        let after = if self.spans.is_empty() { "" } else { ":" };
        writeln!(writer, "{}: {}{}", ty, self.msg, after)?;
        for span in &self.spans {
            let file = map.get(&span.file).expect("unknown file id");
            writeln!(writer, "{:?}", &file.path)?;
            span.write_for(writer, &file.text)?;
        }
        Ok(())
    }
}

impl CompilerOutput {
    pub fn new() -> Self {
        Self {
            errs: Vec::new(),
            hints: Vec::new(),
            file_map: FileMap::new(),
        }
    }
    pub fn err(&mut self, msg: CompilerMsg) {
        self.errs.push(msg);
    }
    pub fn hint(&mut self, msg: CompilerMsg) {
        self.hints.push(msg);
    }
    pub fn write_to(&self, out: &mut impl std::io::Write) {
        for err in &self.errs {
            err.write_to("error", out, &self.file_map).unwrap();
        }
        for hint in &self.hints {
            hint.write_to("hint", out, &self.file_map).unwrap();
        }
    }
}
