use super::{token::{FileRegion, TokenInstance}, FilePos};

#[derive(Debug)]
pub struct ParserError {
    pub msg: String,
    pub regions: Vec<FileRegion>,
}

impl ParserError {
    pub fn from_instances(instances: &[&TokenInstance], msg: String) -> Self {
        ParserError {
            msg,
            regions: instances.iter().map(|i| i.region).collect(),
        }
    }
    pub fn from_msg(msg: String) -> Self {
        Self {
            msg,
            regions: Vec::new(),
        }
    }
    pub fn at(pos: FilePos, msg: String) -> Self {
        Self {
            msg,
            regions: vec![FileRegion {
                start: pos,
                end: pos,
            }],
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
        let after = if self.regions.is_empty() { "" } else { ":" };
        writeln!(writer, "error: {}{}", self.msg, after)?;
        for reg in &self.regions {
            reg.write_for(writer, file)?;
        }
        Ok(())
    }
}

