use super::*;

pub fn opstr(op: Funct3, funct: Funct7) -> &'static str {
    match (op, funct) {
        (op32i::SLT, op32i::FUNCT7) => "slt",
        (op32i::SLTU, op32i::FUNCT7) => "sltu",
        (op32i::XOR, op32i::FUNCT7) => "xor",
        (op32i::OR, op32i::FUNCT7) => "or",
        (op32i::AND, op32i::FUNCT7) => "and",

        (op32i::ADD, op32i::F7ADD) => "add",
        (op32i::ADD, op32i::F7SUB) => "sub",
        (op32i::SL, op32i::LOGICAL) => "sll",
        (op32i::SR, op32i::LOGICAL) => "srl",
        (op32i::SR, op32i::ARITHMETIC) => "sra",

        (op32m::MUL, op32m::FUNCT7) => "mul",
        (op32m::MULH, op32m::FUNCT7) => "mulh",
        (op32m::MULHSU, op32m::FUNCT7) => "mulhsu",
        (op32m::MULHU, op32m::FUNCT7) => "mulhu",
        (op32m::DIV, op32m::FUNCT7) => "div",
        (op32m::DIVU, op32m::FUNCT7) => "divu",
        (op32m::REM, op32m::FUNCT7) => "rem",
        (op32m::REMU, op32m::FUNCT7) => "remu",
        _ => "unknown",
    }
}
