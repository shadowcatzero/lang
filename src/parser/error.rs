use crate::token::{FileRegion, TokenInstance};

#[derive(Debug)]
pub struct ParserError {
    pub msg: String,
    pub regions: Vec<FileRegion>,
}

impl ParserError {
    pub fn from_instances(instances: &[&TokenInstance], msg: String) -> Self {
        ParserError {
            msg,
            regions: instances.iter().map(|i| i.loc).collect(),
        }
    }
    pub fn from_msg(msg: String) -> Self {
        Self {
            msg,
            regions: Vec::new(),
        }
    }
}

pub fn unexpected_token<T>(inst: &TokenInstance, expected: &str) -> Result<T, ParserError> {
    let t = &inst.token;
    Err(ParserError::from_instances(
        &[inst],
        format!("Unexpected token {t:?}; expected {expected}"),
    ))
}

pub fn unexpected_end() -> ParserError {
    ParserError::from_msg("Unexpected end of input".to_string())
}

const BEFORE: usize = 1;
const AFTER: usize = 1;

pub fn print_error(err: ParserError, file: &str) {
    println!("error: {}:", err.msg);
    for reg in err.regions {
        print_region(file, reg);
    }
}

pub fn print_region(file: &str, reg: FileRegion) {
    let start = reg.start.line.saturating_sub(BEFORE);
    let num_before = reg.start.line - start;
    let mut lines = file.lines().skip(start);
    let len = reg.end.col - reg.start.col + 1;
    let width = format!("{}", reg.end.line + AFTER).len();
    for i in 0..num_before + 1 {
        println!("{:>width$} | {}", start + i, lines.next().unwrap());
    }
    println!(
        "{} | {}",
        " ".repeat(width),
        " ".repeat(reg.start.col) + &"^".repeat(len)
    );
    for i in 0..AFTER {
        if let Some(next) = lines.next() {
            println!("{:>width$} | {}", reg.end.line + i + 1, next);
        }
    }
}
