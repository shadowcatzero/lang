#![feature(box_patterns)]
#![feature(try_trait_v2)]

mod util;
mod compiler;
mod ir;
mod parser;

fn main() {
    parser::main();
    // compiler::main();
}
