#![feature(box_patterns)]
#![feature(const_unbounded_shifts)]
#![feature(unbounded_shifts)]

mod util;
mod compiler;
mod parser;

fn main() {
    compiler::main();
}
