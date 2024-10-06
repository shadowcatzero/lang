use super::{FilePos, FileRegion};

#[derive(Debug)]
pub struct ParserError {
    pub msg: String,
    pub regions: Vec<FileRegion>,
}

impl ParserError {
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
        Self::from_msg("Unexpected end of input".to_string())
    }
}

const BEFORE: usize = 1;
const AFTER: usize = 1;

pub fn print_error(err: ParserError, file: &str) {
    let after = if err.regions.is_empty() {""} else {":"};
    println!("error: {}{}", err.msg, after);
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
