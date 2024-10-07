use std::io::{stdout, BufRead, BufReader};

mod parser;

use parser::{Module, Node, NodeContainer, Statement, TokenCursor};

pub fn parse_file(file: &str) {
    let node = Node::<Module>::parse(&mut TokenCursor::from(file));
    match node.inner {
        Err(err) => err.write_for(&mut stdout(), file).unwrap(),
        Ok(module) => {
            println!("{module:#?}");
            print_errors(module.children(), file)
        },
    };
}

pub fn print_errors(nodes: Vec<Node<Box<dyn NodeContainer>>>, file: &str) {
    for node in &nodes {
        if let Err(err) = &node.inner {
            err.write_for(&mut stdout(), file).unwrap();
        }
    }
    for node in nodes {
        print_errors(node.children(), file)
    }
}

pub fn run_stdin() {
    for line in BufReader::new(std::io::stdin()).lines() {
        let str = &line.expect("failed to read line");
        let mut cursor = TokenCursor::from(&str[..]);
        let out = &mut stdout();
        match Node::<Statement>::parse(&mut cursor).inner {
            Ok(expr) => println!("{:?}", expr),
            Err(err) => err.write_for(out, str).unwrap(),
        }
    }
}
