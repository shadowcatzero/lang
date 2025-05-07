use super::{arch::riscv64::RegRef, IdentID};

#[derive(Clone)]
pub struct IRAsmInstruction {
    op: String,
    args: Vec<RegRef<IdentID, String>>,
}
