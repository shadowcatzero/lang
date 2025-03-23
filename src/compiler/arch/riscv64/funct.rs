use crate::{ir::Len, util::Bits32};

pub type Funct3 = Bits32<2, 0>;
pub type Funct7 = Bits32<6, 0>;

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum Op {
    Add = 0b000,
    Sl = 0b001,
    Slt = 0b010,
    Sltu = 0b011,
    Xor = 0b100,
    Sr = 0b101,
    Or = 0b110,
    And = 0b111,
}

impl Op {
    pub const fn code(&self) -> Funct3 {
        Funct3::new(*self as u32)
    }
    pub const fn str(&self) -> &str {
        match self {
            Op::Add => "add",
            Op::Sl => "sl",
            Op::Slt => "slt",
            Op::Sltu => "sltu",
            Op::Xor => "xor",
            Op::Sr => "sr",
            Op::Or => "or",
            Op::And => "and",
        }
    }
    pub fn fstr(&self, funct: Funct7) -> String {
        match (self, funct) {
            (Op::Add, funct7::ADD) => "add",
            (Op::Add, funct7::SUB) => "sub",
            (Op::Sl, funct7::LOGICAL) => "sll",
            (Op::Sr, funct7::LOGICAL) => "srl",
            (Op::Sr, funct7::ARITHMETIC) => "sra",
            other => return self.str().to_string() + "?"
        }.to_string()
    }
}

pub mod funct7 {
    use super::Funct7;

    pub const LOGICAL: Funct7 = Funct7::new(0b0000000);
    pub const ARITHMETIC: Funct7 = Funct7::new(0b0100000);
    pub const ADD: Funct7 = Funct7::new(0b0000000);
    pub const SUB: Funct7 = Funct7::new(0b0100000);
}

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum Width {
    B = 0b000,
    H = 0b001,
    W = 0b010,
    D = 0b011,
    BU = 0b100,
    HU = 0b101,
    WU = 0b110,
}

impl Width {
    pub const MAIN: [Self; 4] = [Self::B, Self::H, Self::W, Self::D];

    pub const fn code(&self) -> Funct3 {
        Funct3::new(*self as u32)
    }
    pub const fn len(&self) -> Len {
        match self {
            Width::B => 1,
            Width::H => 2,
            Width::W => 4,
            Width::D => 8,
            Width::BU => 1,
            Width::HU => 2,
            Width::WU => 4,
        }
    }
    pub const fn str(&self) -> &str {
        match self {
            Width::B => "b",
            Width::H => "h",
            Width::W => "w",
            Width::D => "d",
            Width::BU => "bu",
            Width::HU => "hu",
            Width::WU => "wu",
        }
    }
}
