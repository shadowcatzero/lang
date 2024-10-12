pub const fn u(x: i32) -> u32 {
    unsafe { std::mem::transmute(x) }
}

pub const fn base_mask(len: u8) -> u32 {
    (2 << len) - 1
}

pub const fn mask(h: u8, l: u8) -> u32 {
    base_mask(h - l) << l
}

#[derive(Debug, Clone, Copy)]
pub struct Bits32<const H: u8, const L: u8>(u32);
#[derive(Debug, Clone, Copy)]
pub struct BitsI32<const H: u8, const L: u8>(u32);

impl<const H: u8, const L: u8> Bits32<H, L> {
    pub const fn new(val: u32) -> Self {
        let lsh = 31 - H;
        let rsh = lsh + L;
        debug_assert!(((val << lsh) >> rsh) == (val >> L));
        Self(val)
    }
    pub const fn val(&self) -> u32 {
        self.0
    }
    pub const fn bit(&self, i: u8) -> u32 {
        (self.0 >> i) & 1
    }
    pub const fn bits(&self, h: u8, l: u8) -> u32 {
        (self.0 >> l) & base_mask(h - l)
    }
}

impl<const H: u8, const L: u8> BitsI32<H, L> {
    pub const fn new(val: i32) -> Self {
        let lsh = 31 - H;
        let rsh = lsh + L;
        assert!(((val << lsh) >> rsh) == (val >> L));
        Self(u(val) & mask(H, L))
    }
    pub const fn tryy(val: i32) -> Option<Self> {
        let lsh = 31 - H;
        let rsh = lsh + L;
        if ((val << lsh) >> rsh) == (val >> L) {
            Some(Self(u(val) & mask(H, L)))
        } else {
            None
        }
    }
    pub const fn to_u(self) -> Bits32<H, L> {
        Bits32(self.0)
    }
}

// I hate (love) rust https://github.com/rust-lang/rust-project-goals/issues/106
// pub struct Bits<
//     T: Shl<u8, Output = T> + Shr<u8, Output = T>,
//     const S: bool,
//     const H: u8,
//     const L: u8,
// >(T);
// pub struct U32Bits<const H: u32, const L: u32>(u32);
//
// impl<T: const Shl<u8, Output = T> + const Shr<u8, Output = T>, const S: bool, const H: u8, const L: u8>
//     Bits<T, S, H, L>
// {
//     pub const fn new(val: T) -> Self {
//         assert!(in_bit_range(val, H, L));
//         Self(val + L)
//     }
// }
