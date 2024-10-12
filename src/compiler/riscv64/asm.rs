use crate::compiler::program::Instr;

use super::*;

pub enum AsmInstruction {
    Addi(Reg, Reg, i32),
    La(Reg, String),
    Jal(Reg, i32),
    Jala(String),
    Ja(String),
    Ret,
    Ecall,
}

impl Instr for AsmInstruction {
    fn push(
        &self,
        data: &mut Vec<u8>,
        sym_map: &std::collections::HashMap<String, u64>,
        pos: u64,
    ) -> Option<String> {
        match self {
            Self::Addi(dest, src, imm) => {
                data.extend(addi(*dest, *src, BitsI32::new(*imm)).to_le_bytes());
            }
            Self::La(dest, sym) => {
                if let Some(addr) = sym_map.get(sym) {
                    let offset = *addr as i32 - pos as i32;
                    data.extend(auipc(*dest, BitsI32::new(0)).to_le_bytes());
                    data.extend(addi(*dest, *dest, BitsI32::new(offset)).to_le_bytes());
                } else {
                    return Some(sym.to_string());
                }
            }
            Self::Jal(dest, offset) => data.extend(jal(*dest, BitsI32::new(*offset)).to_le_bytes()),
            Self::Ja(sym) => {
                if let Some(addr) = sym_map.get(sym) {
                    let offset = *addr as i32 - pos as i32;
                    data.extend(j(BitsI32::new(offset)).to_le_bytes());
                } else {
                    return Some(sym.to_string());
                }
            }
            Self::Jala(sym) => {
                if let Some(addr) = sym_map.get(sym) {
                    let offset = *addr as i32 - pos as i32;
                    data.extend(jal(ra, BitsI32::new(offset)).to_le_bytes());
                } else {
                    return Some(sym.to_string());
                }
            }
            Self::Ret => data.extend(ret().to_le_bytes()),
            Self::Ecall => data.extend(ecall().to_le_bytes()),
        }
        None
    }
}
