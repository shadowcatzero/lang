use std::io::{stdout, BufRead, BufReader};

mod cursor;
mod error;
mod node;
mod nodes;
mod parse;
mod token;

pub use cursor::*;
pub use error::*;
pub use node::*;
pub use nodes::*;
pub use parse::*;
pub use token::*;

use crate::ir::Namespace;

pub fn parse_file(file: &str) {
    let mut errors = ParserErrors::new();
    let res = Module::parse_node(&mut TokenCursor::from(file), &mut errors);
    println!("{:?}", res.node);
    let out = &mut stdout();
    if let Some(module) = res.node.as_ref() {
        let mut namespace = Namespace::new();
        module.lower(&mut namespace, &mut errors);
        println!("{:?}", namespace);
    }
    for err in errors.errs {
        err.write_for(out, file).unwrap();
    }
}

pub fn run_stdin() {
    for line in BufReader::new(std::io::stdin()).lines() {
        let mut errors = ParserErrors::new();
        let str = &line.expect("failed to read line");
        let mut cursor = TokenCursor::from(&str[..]);
        if let Some(expr) = Statement::parse_node(&mut cursor, &mut errors).node.as_ref() {
            if cursor.next().is_none() {
                println!("{:?}", expr);
            } else {
                println!("uhhhh ehehe");
            }
        }
        let out = &mut stdout();
        for err in errors.errs {
            err.write_for(out, str).unwrap();
        }
    }
}
