use std::{
    ffi::OsStr,
    io::{BufRead, BufReader},
};

mod parser;
mod token;
mod util;

use parser::{print_error, Expr, Module, TokenCursor};

fn main() {
    let arg = std::env::args_os().nth(1);
    if let Some(file) = arg {
        run_file(&file);
    } else {
        run_stdin();
    }
}

fn run_file(path: &OsStr) {
    let file = std::fs::read_to_string(path).expect("failed to read file");
    let tokens = token::parse(&file).unwrap();
    match Module::parse(&mut TokenCursor::from(tokens.as_slice())) {
        Err(err) => print_error(err, &file),
        Ok(module) => println!("{module:#?}"),
    }
}

fn run_stdin() {
    for line in BufReader::new(std::io::stdin()).lines() {
        let str = &line.expect("failed to read line");
        let tokens = token::parse(str).unwrap();
        println!(
            "{:?}",
            Expr::parse(&mut TokenCursor::from(tokens.as_slice()))
        );
    }
}
