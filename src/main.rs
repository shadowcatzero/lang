#![feature(box_patterns)]

mod util;
mod compiler;
mod ir;
mod parser;

fn main() {
    parser::main();
    // compiler::main();
}
