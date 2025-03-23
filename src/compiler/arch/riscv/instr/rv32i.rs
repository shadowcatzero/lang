use crate::{compiler::arch::riscv::Reg, util::Bits32};

use super::{opr, Funct3, Funct7, RawInstruction};

pub mod op32i {
    use super::*;

    pub const ADD: Funct3 = Funct3::new(0b000);
    pub const SL: Funct3 = Funct3::new(0b001);
    pub const SLT: Funct3 = Funct3::new(0b010);
    pub const SLTU: Funct3 = Funct3::new(0b011);
    pub const XOR: Funct3 = Funct3::new(0b100);
    pub const SR: Funct3 = Funct3::new(0b101);
    pub const OR: Funct3 = Funct3::new(0b110);
    pub const AND: Funct3 = Funct3::new(0b111);

    pub const LOGICAL: Funct7 = Funct7::new(0b0000000);
    pub const ARITHMETIC: Funct7 = Funct7::new(0b0100000);
    pub const F7ADD: Funct7 = Funct7::new(0b0000000);
    pub const F7SUB: Funct7 = Funct7::new(0b0100000);

    pub const FUNCT7: Funct7 = Funct7::new(0b0000000);
}

pub mod width {
    use crate::ir::Len;

    use super::*;
    pub const MAIN: [Funct3; 4] = [B, H, W, D];

    pub const B: Funct3 = Funct3::new(0b000);
    pub const H: Funct3 = Funct3::new(0b001);
    pub const W: Funct3 = Funct3::new(0b010);
    pub const D: Funct3 = Funct3::new(0b011);
    pub const BU: Funct3 = Funct3::new(0b100);
    pub const HU: Funct3 = Funct3::new(0b101);
    pub const WU: Funct3 = Funct3::new(0b110);

    pub const fn str(w: Funct3) -> &'static str {
        match w {
            B => "b",
            H => "h",
            W => "w",
            D => "d",
            BU => "bu",
            HU => "hu",
            WU => "wu",
            _ => unreachable!(),
        }
    }

    pub const fn len(w: Funct3) -> Len {
        match w {
            B => 1,
            H => 2,
            W => 4,
            D => 8,
            BU => 1,
            HU => 2,
            WU => 4,
            _ => unreachable!(),
        }
    }
}

pub fn opr32i(op: Funct3, dest: Reg, src1: Reg, src2: Reg) -> RawInstruction {
    opr(op, Bits32::new(0), dest, src1, src2)
}
