use std::io::{stdout, BufRead, BufReader};

mod parser;

use parser::{Module, Statement, TokenCursor};

pub fn parse_file(file: &str) {
    match Module::parse(&mut TokenCursor::from(file)) {
        Err(err) => err.write_for(&mut stdout(), file).unwrap(),
        Ok(module) => println!("{module:#?}"),
    }
}

pub fn run_stdin() {
    for line in BufReader::new(std::io::stdin()).lines() {
        let str = &line.expect("failed to read line");
        let mut cursor = TokenCursor::from(&str[..]);
        let out = &mut stdout();
        match Statement::parse(&mut cursor) {
            Ok(expr) => println!("{:?}", expr),
            Err(err) => err.write_for(out, str).unwrap(),
        }
    }
}
