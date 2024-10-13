use std::io::{stdout, BufRead, BufReader};

mod body;
mod cursor;
mod error;
mod expr;
mod module;
mod node;
mod op;
mod token;
mod val;
mod statement;
mod func;

pub use body::*;
pub use cursor::*;
pub use error::*;
pub use expr::*;
pub use module::*;
pub use node::*;
pub use op::*;
use token::*;
pub use val::*;
pub use statement::*;

pub fn parse_file(file: &str) {
    let mut errors = ParserErrors::new();
    let node = Module::parse_node(&mut TokenCursor::from(file), &mut errors);
    if errors.errs.is_empty() {
        let module = node.resolve().expect("what");
    }
    let out = &mut stdout();
    for err in errors.errs {
        err.write_for(out, file).unwrap();
    }
}

pub fn run_stdin() {
    for line in BufReader::new(std::io::stdin()).lines() {
        let mut errors = ParserErrors::new();
        let str = &line.expect("failed to read line");
        let mut cursor = TokenCursor::from(&str[..]);
        if let Ok(expr) = Statement::parse_node(&mut cursor, &mut errors).as_ref() {
            println!("{:?}", expr);
        }
        let out = &mut stdout();
        for err in errors.errs {
            err.write_for(out, str).unwrap();
        }
    }
}
