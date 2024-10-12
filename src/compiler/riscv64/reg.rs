#![allow(non_upper_case_globals)]

#[derive(Debug, Clone, Copy)]
pub struct Reg(u8);

/// hard wired 0
pub const zero: Reg = Reg(0);
/// return address
pub const ra: Reg = Reg(1);
/// stack pointer
pub const sp: Reg = Reg(2);
/// global pointer
pub const gp: Reg = Reg(3);
/// thread pointer
pub const tp: Reg = Reg(4);
/// temp / alternate link
pub const t0: Reg = Reg(5);
pub const t1: Reg = Reg(6);
pub const t2: Reg = Reg(7);

pub const fp: Reg = Reg(8);
pub const s0: Reg = Reg(8);
pub const s1: Reg = Reg(9);

pub const a0: Reg = Reg(10);
pub const a1: Reg = Reg(11);
pub const a2: Reg = Reg(12);
pub const a3: Reg = Reg(13);
pub const a4: Reg = Reg(14);
pub const a5: Reg = Reg(15);
pub const a6: Reg = Reg(16);
pub const a7: Reg = Reg(17);

pub const s2: Reg = Reg(18);
pub const s3: Reg = Reg(19);
pub const s4: Reg = Reg(20);
pub const s5: Reg = Reg(21);
pub const s6: Reg = Reg(22);
pub const s7: Reg = Reg(23);
pub const s8: Reg = Reg(24);
pub const s9: Reg = Reg(25);
pub const s10: Reg = Reg(26);
pub const s11: Reg = Reg(27);

pub const t3: Reg = Reg(28);
pub const t4: Reg = Reg(29);
pub const t5: Reg = Reg(30);
pub const t6: Reg = Reg(31);



pub const ft0: Reg = Reg(0);
pub const ft1: Reg = Reg(1);
pub const ft2: Reg = Reg(2);
pub const ft3: Reg = Reg(3);
pub const ft4: Reg = Reg(4);
pub const ft5: Reg = Reg(5);
pub const ft6: Reg = Reg(6);
pub const ft7: Reg = Reg(7);

pub const fs0: Reg = Reg(8);
pub const fs1: Reg = Reg(9);

pub const fa0: Reg = Reg(10);
pub const fa1: Reg = Reg(11);
pub const fa2: Reg = Reg(12);
pub const fa3: Reg = Reg(13);
pub const fa4: Reg = Reg(14);
pub const fa5: Reg = Reg(15);
pub const fa6: Reg = Reg(16);
pub const fa7: Reg = Reg(17);

pub const fs2: Reg = Reg(18);
pub const fs3: Reg = Reg(19);
pub const fs4: Reg = Reg(20);
pub const fs5: Reg = Reg(21);
pub const fs6: Reg = Reg(22);
pub const fs7: Reg = Reg(23);
pub const fs8: Reg = Reg(24);
pub const fs9: Reg = Reg(25);
pub const fs10: Reg = Reg(26);
pub const fs11: Reg = Reg(27);

pub const ft8: Reg = Reg(28);
pub const ft9: Reg = Reg(29);
pub const ft10: Reg = Reg(30);
pub const ft11: Reg = Reg(31);

impl Reg {
    #[inline]
    pub const fn val(&self) -> u32 {
        self.0 as u32
    }
}
