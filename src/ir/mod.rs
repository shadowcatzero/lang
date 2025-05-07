//! the IR is split into 2 layers: upper and lower
//! upper handles all of the main language features like types,
//! and the lower is a very concrete format that can be easily
//! translated to assembly and will probably also include
//! the majority of optimization, but not sure

mod upper;
mod lower;
mod id;
mod asm;
pub mod arch;

pub use upper::*;
pub use lower::*;
pub use id::*;

