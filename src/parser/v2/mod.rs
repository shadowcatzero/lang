use std::io::{BufRead, BufReader};

mod body;
mod cursor;
mod error;
mod expr;
mod module;
mod util;

pub use body::*;
pub use cursor::*;
pub use error::*;
pub use expr::*;
pub use module::*;

pub fn parse_file(file: &str) {
    match Module::parse(&mut CharCursor::from(file)) {
        Err(err) => print_error(err, file),
        Ok(module) => println!("{module:#?}"),
    }
}

pub fn run_stdin() {
    for line in BufReader::new(std::io::stdin()).lines() {
        let str = &line.expect("failed to read line");
        let mut cursor = CharCursor::from(&str[..]);
        match Statement::parse(&mut cursor) {
            Ok(expr) => println!("{:?}", expr),
            Err(err) => print_error(err, str),
        }
    }
}
