use crate::{compiler::arch::riscv::Reg, util::Bits32};

use super::*;

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

pub mod branch {
    use super::*;
    pub const EQ: Funct3 = Funct3::new(0b000);
    pub const NE: Funct3 = Funct3::new(0b001);
    pub const LT: Funct3 = Funct3::new(0b100);
    pub const GE: Funct3 = Funct3::new(0b101);
    pub const LTU: Funct3 = Funct3::new(0b110);
    pub const GEU: Funct3 = Funct3::new(0b111);

    pub fn str(f: Funct3) -> &'static str {
        match f {
            EQ => "eq",
            NE => "ne",
            LT => "lt",
            GE => "ge",
            LTU => "ltu",
            GEU => "geu",
            _ => "?",
        }
    }
}

pub const fn ecall() -> RawInstruction {
    i_type(Bits32::new(0), zero, Bits32::new(0), zero, SYSTEM)
}
pub const fn ebreak() -> RawInstruction {
    i_type(Bits32::new(1), zero, Bits32::new(0), zero, SYSTEM)
}
pub const fn auipc(dest: Reg, imm: BitsI32<31, 12>) -> RawInstruction {
    u_type(imm.to_u(), dest, AUIPC)
}

pub const fn load(width: Funct3, dest: Reg, offset: BitsI32<11, 0>, base: Reg) -> RawInstruction {
    i_type(offset.to_u(), base, width, dest, LOAD)
}

pub const fn store(width: Funct3, src: Reg, offset: BitsI32<11, 0>, base: Reg) -> RawInstruction {
    s_type(src, base, width, offset.to_u(), STORE)
}

pub const fn jal(dest: Reg, offset: BitsI32<20, 1>) -> RawInstruction {
    j_type(offset.to_u(), dest, JAL)
}
pub const fn jalr(dest: Reg, offset: BitsI32<11, 0>, base: Reg) -> RawInstruction {
    i_type(offset.to_u(), base, Bits32::new(0), dest, JALR)
}

pub const fn j(offset: BitsI32<20, 1>) -> RawInstruction {
    jal(zero, offset)
}
pub const fn ret() -> RawInstruction {
    jalr(zero, BitsI32::new(0), ra)
}

pub const fn branch(typ: Funct3, left: Reg, right: Reg, offset: BitsI32<12, 1>) -> RawInstruction {
    b_type(right, left, typ, offset.to_u(), BRANCH)
}
