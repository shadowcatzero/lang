pub const OPCODE_MASK: u32 = 0b1111111;

pub const SYSTEM: u32 = 0b1110011;
pub const LOAD  : u32 = 0b0000011;
pub const STORE : u32 = 0b0100011;
pub const AUIPC : u32 = 0b0010111;
pub const IMM_OP: u32 = 0b0010011;
pub const OP    : u32 = 0b0110011;
pub const JAL   : u32 = 0b1101111;
pub const JALR  : u32 = 0b1100111;
