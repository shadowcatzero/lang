use crate::{compiler::program::{Addr, Instr, SymTable}, ir::Symbol};

use super::*;

#[derive(Debug, Clone)]
pub enum LinkerInstruction {
    Add { dest: Reg, src1: Reg, src2: Reg },
    Addi { dest: Reg, src: Reg, imm: i32 },
    Andi { dest: Reg, src: Reg, imm: i32 },
    Slli { dest: Reg, src: Reg, imm: i32 },
    Srli { dest: Reg, src: Reg, imm: i32 },
    Sd { src: Reg, offset: i32, base: Reg },
    Ld { dest: Reg, offset: i32, base: Reg },
    Mv { dest: Reg, src: Reg },
    La { dest: Reg, src: Symbol },
    Jal { dest: Reg, offset: i32 },
    Call(Symbol),
    J(Symbol),
    Ret,
    Ecall,
    Li { dest: Reg, imm: i64 },
}

impl Instr for LinkerInstruction {
    fn push(
        &self,
        data: &mut Vec<u8>,
        sym_map: &SymTable,
        pos: Addr,
        missing: bool,
    ) -> Option<Symbol> {
        let last = match self {
            Self::Add { dest, src1, src2 } => add(*dest, *src1, *src2),
            Self::Addi { dest, src, imm } => addi(*dest, *src, BitsI32::new(*imm)),
            Self::Andi { dest, src, imm } => andi(*dest, *src, BitsI32::new(*imm)),
            Self::Slli { dest, src, imm } => slli(*dest, *src, BitsI32::new(*imm)),
            Self::Srli { dest, src, imm } => srli(*dest, *src, BitsI32::new(*imm)),
            Self::Sd { src, offset, base } => sd(*src, BitsI32::new(*offset), *base),
            Self::Ld { dest, offset, base } => ld(*dest, BitsI32::new(*offset), *base),
            Self::Mv { dest, src } => addi(*dest, *src, BitsI32::new(0)),
            Self::La { dest, src } => {
                if let Some(addr) = sym_map.get(*src) {
                    let offset = addr.val() as i32 - pos.val() as i32;
                    data.extend(auipc(*dest, BitsI32::new(0)).to_le_bytes());
                    addi(*dest, *dest, BitsI32::new(offset))
                } else {
                    data.extend_from_slice(&[0; 2 * 4]);
                    return Some(*src);
                }
            }
            Self::Jal { dest, offset } => jal(*dest, BitsI32::new(*offset)),
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
            Self::Li { dest, imm } => addi(*dest, zero, BitsI32::new(*imm as i32)),
        };
        data.extend(last.to_le_bytes());
        None
    }
}
