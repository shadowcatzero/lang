#![feature(box_patterns)]
#![feature(try_trait_v2)]

mod util;
mod compiler;
mod ir;
mod parser;

fn main() {
    let arg = std::env::args_os().nth(1);
    if let Some(path) = arg {
        let file = std::fs::read_to_string(path).expect("failed to read file");
        println!("{file}");
        parser::parse_file(&file);
    } else {
        parser::run_stdin();
    }
    // compiler::main();
}
