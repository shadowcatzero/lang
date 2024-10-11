use std::io::{stdout, BufRead, BufReader};

mod body;
mod cursor;
mod error;
mod expr;
mod module;
mod node;
mod token;
mod val;

pub use body::*;
pub use cursor::*;
pub use error::*;
pub use expr::*;
pub use module::*;
pub use node::*;
pub use val::*;
use token::*;

pub fn parse_file(file: &str) {
    let mut errors = ParserErrors::new();
    let node = Node::<Module>::parse(&mut TokenCursor::from(file), &mut errors);
    if let Ok(module) = node.as_ref() {
        println!("{module:#?}");
    };
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
        if let Ok(expr) = Node::<Statement>::parse(&mut cursor, &mut errors).as_ref() {
            println!("{:?}", expr);
        }
        let out = &mut stdout();
        for err in errors.errs {
            err.write_for(out, str).unwrap();
        }
    }
}
