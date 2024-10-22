#![feature(box_patterns)]
#![feature(try_trait_v2)]

use std::io::{stdout, BufRead, BufReader};

use ir::Namespace;
use parser::{Module, NodeParsable, ParserOutput, Statement, TokenCursor};

mod compiler;
mod ir;
mod parser;
mod util;

fn main() {
    let arg = std::env::args_os().nth(1);
    if let Some(path) = arg {
        let file = std::fs::read_to_string(path).expect("failed to read file");
        run_file(&file);
    } else {
        run_stdin();
    }
    // compiler::main();
}

fn run_file(file: &str) {
    let mut output = ParserOutput::new();
    let res = Module::parse_node(&mut TokenCursor::from(file), &mut output);
    println!("{:?}", res.node);
    if let Some(module) = res.node.as_ref() {
        let mut namespace = Namespace::new();
        module.lower(&mut namespace.push(), &mut output);
        println!("{:?}", namespace.fns);
    }
    output.write_for(&mut stdout(), file);
}

pub fn run_stdin() {
    for line in BufReader::new(std::io::stdin()).lines() {
        let mut output = ParserOutput::new();
        let str = &line.expect("failed to read line");
        let mut cursor = TokenCursor::from(&str[..]);
        if let Some(expr) = Statement::parse_node(&mut cursor, &mut output)
            .node
            .as_ref()
        {
            if cursor.next().is_none() {
                println!("{:?}", expr);
            } else {
                println!("uhhhh ehehe");
            }
        }
        output.write_for(&mut stdout(), str);
    }
}
