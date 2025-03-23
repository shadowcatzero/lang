use super::{Funct3, Funct7};

pub mod op32m {
    use super::*;
    pub const MUL: Funct3 = Funct3::new(0b000);
    pub const MULH: Funct3 = Funct3::new(0b001);
    pub const MULHSU: Funct3 = Funct3::new(0b010);
    pub const MULHU: Funct3 = Funct3::new(0b011);
    pub const DIV: Funct3 = Funct3::new(0b100);
    pub const DIVU: Funct3 = Funct3::new(0b101);
    pub const REM: Funct3 = Funct3::new(0b110);
    pub const REMU: Funct3 = Funct3::new(0b111);

    pub const FUNCT7: Funct7 = Funct7::new(0b0000001);
}

