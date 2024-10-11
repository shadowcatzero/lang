pub mod op {
    pub const ADD : u32 = 0b000;
    pub const SLL : u32 = 0b001;
    pub const SLT : u32 = 0b010;
    pub const SLTU: u32 = 0b011;
    pub const XOR : u32 = 0b100;
    pub const SR  : u32 = 0b101;
    pub const OR  : u32 = 0b110;
    pub const AND : u32 = 0b111;
}

pub mod width {
    pub const B : u32 = 0b000;
    pub const H : u32 = 0b001;
    pub const W : u32 = 0b010;
    pub const D : u32 = 0b011;
    pub const BU: u32 = 0b100;
    pub const HU: u32 = 0b101;
    pub const WU: u32 = 0b110;
}
