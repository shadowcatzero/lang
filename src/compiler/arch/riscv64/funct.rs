use super::Funct3;

pub mod op {
    use super::*;
    pub const ADD : Funct3 = Funct3::new(0b000);
    pub const SLL : Funct3 = Funct3::new(0b001);
    pub const SLT : Funct3 = Funct3::new(0b010);
    pub const SLTU: Funct3 = Funct3::new(0b011);
    pub const XOR : Funct3 = Funct3::new(0b100);
    pub const SR  : Funct3 = Funct3::new(0b101);
    pub const OR  : Funct3 = Funct3::new(0b110);
    pub const AND : Funct3 = Funct3::new(0b111);
}

pub mod width {
    use super::*;
    pub const B : Funct3 = Funct3::new(0b000);
    pub const H : Funct3 = Funct3::new(0b001);
    pub const W : Funct3 = Funct3::new(0b010);
    pub const D : Funct3 = Funct3::new(0b011);
    pub const BU: Funct3 = Funct3::new(0b100);
    pub const HU: Funct3 = Funct3::new(0b101);
    pub const WU: Funct3 = Funct3::new(0b110);
}
