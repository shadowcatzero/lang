use std::io::{stdout, BufRead, BufReader};

mod parser;

use parser::{Module, Node, ParserErrors, Statement, TokenCursor};

pub fn parse_file(file: &str) {
    let mut errors = ParserErrors::new();
    let node = Node::<Module>::parse(&mut TokenCursor::from(file), &mut errors);
    if let Ok(module) = node.as_ref() {
        println!("{module:#?}");
    };
    for err in errors.errs {
        err.write_for(&mut stdout(), file).unwrap();
    }
}

pub fn run_stdin() {
    for line in BufReader::new(std::io::stdin()).lines() {
        let mut errors = ParserErrors::new();
        let str = &line.expect("failed to read line");
        let mut cursor = TokenCursor::from(&str[..]);
        let out = &mut stdout();
        if let Ok(expr) = Node::<Statement>::parse(&mut cursor, &mut errors).as_ref() {
            println!("{:?}", expr);
        }
        for err in errors.errs {
            err.write_for(&mut stdout(), str).unwrap();
        }
    }
}
