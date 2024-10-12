use crate::compiler::program::{Addr, Instr, SymTable, Symbol};

use super::*;

pub enum AsmInstruction {
    Addi(Reg, Reg, i32),
    La(Reg, Symbol),
    Jal(Reg, i32),
    Call(Symbol),
    J(Symbol),
    Ret,
    Ecall,
    Li(Reg, i32),
}

impl Instr for AsmInstruction {
    fn push(&self, data: &mut Vec<u8>, sym_map: &SymTable, pos: Addr, missing: bool) -> Option<Symbol> {
        let last = match self {
            Self::Addi(dest, src, imm) => addi(*dest, *src, BitsI32::new(*imm)),
            Self::La(dest, sym) => {
                if let Some(addr) = sym_map.get(*sym) {
                    let offset = addr.val() as i32 - pos.val() as i32;
                    data.extend(auipc(*dest, BitsI32::new(0)).to_le_bytes());
                    addi(*dest, *dest, BitsI32::new(offset))
                } else {
                    data.extend_from_slice(&[0; 2 * 4]);
                    return Some(*sym);
                }
            }
            Self::Jal(dest, offset) => jal(*dest, BitsI32::new(*offset)),
            Self::J(sym) => {
                if let Some(addr) = sym_map.get(*sym) {
                    let offset = addr.val() as i32 - pos.val() as i32;
                    j(BitsI32::new(offset))
                } else {
                    data.extend_from_slice(&[0; 4]);
                    return Some(*sym);
                }
            }
            Self::Call(sym) => {
                if let Some(addr) = sym_map.get(*sym) {
                    let offset = addr.val() as i32 - pos.val() as i32;
                    jal(ra, BitsI32::new(offset))
                } else {
                    data.extend_from_slice(&[0; 4]);
                    return Some(*sym);
                }
            }
            Self::Ret => ret(),
            Self::Ecall => ecall(),
            Self::Li(reg, val) => addi(*reg, zero, BitsI32::new(*val)),
        };
        data.extend(last.to_le_bytes());
        None
    }
}
