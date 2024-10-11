pub const fn u(x: i32) -> u32 {
    unsafe { std::mem::transmute(x) }
}

pub const fn low_mask(high: u32, low: u32) -> u32 {
    (2u32.unbounded_shl(high - low)).wrapping_sub(1)
}

pub const fn mask(high: u32, low: u32) -> u32 {
    low_mask(high, low).unbounded_shl(low)
}

pub const fn bits(x: i32, high: u32, low: u32) -> u32 {
    let x = u(x);
    x.unbounded_shr(low) & low_mask(high, low)
}

pub const fn bit(x: i32, i: u32) -> u32 {
    let x = u(x);
    x.unbounded_shr(i) & 2u32 << i
}

pub const fn in_bit_range(x: i32, high: u32, low: u32) -> bool {
    if x < 0 {
        if high == low {
            return false;
        }
        (bits(x, high - 1, low) | !mask(high - 1, low)) == u(x)
    } else {
        bits(x, high, low) << low == u(x)
    }
}

// use std::ops::{Add, Shl, Shr};
//
// pub const fn u(x: i32) -> u32 {
//     unsafe { std::mem::transmute(x) }
// }
//
// pub const fn low_mask(high: u8, low: u8) -> u32 {
//     (2u32.unbounded_shl(high - low)).wrapping_sub(1)
// }
//
// pub const fn mask(high: u8, low: u8) -> u32 {
//     low_mask(high, low).unbounded_shl(low)
// }
//
// pub const fn bits(x: i32, high: u8, low: u8) -> u32 {
//     let x = u(x);
//     x.unbounded_shr(low) & low_mask(high, low)
// }
//
// pub const fn bit(x: i32, i: u32) -> u32 {
//     let x = u(x);
//     x.unbounded_shr(i) & 2u32 << i
// }
//
// pub const fn in_bit_range<T: Shl<u8, Output = T> + Shr<u8, Output = T>>(
//     x: T,
//     high: u8,
//     low: u8,
// ) -> bool {
//     if x < 0 {
//         if high == low {
//             return false;
//         }
//         (bits(x, high - 1, low) | !mask(high - 1, low)) == u(x)
//     } else {
//         bits(x, high, low) << low == u(x)
//     }
// }

// pub struct Bits<
//     T: Shl<u8, Output = T> + Shr<u8, Output = T>,
//     const S: bool,
//     const H: u8,
//     const L: u8,
// >(T);
// pub struct U32Bits<const H: u32, const L: u32>(u32);
//
// impl<T: Shl<u8, Output = T> + Shr<u8, Output = T> + Add<u8, Output = T>, const S: bool, const H: u8, const L: u8>
//     Bits<T, S, H, L>
// {
//     pub const fn new(val: T) -> Self {
//         assert!(in_bit_range(val, H, L));
//         Self(val + L)
//     }
// }


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_bits() {
        assert_eq!(bits(0b10111010, 5, 3), 0b111);
        assert_eq!(bits(0b10111010, 7, 5), 0b101);
        assert_eq!(bits(0b10111010, 2, 0), 0b010);
        assert_eq!(bits(0b10111010, 7, 7), 0b1);
        assert_eq!(bits(0b1, 0, 0), 0b1);
        assert_eq!(bits(0b1, 1, 1), 0b0);
    }

    #[test]
    fn range() {
        assert!(!in_bit_range(0b00111100, 5, 3));
        assert!(!in_bit_range(0b00111100, 4, 2));
        assert!(in_bit_range(0b000111100, 5, 2));
        assert!(in_bit_range(0b000000001, 0, 0));
        assert!(in_bit_range(0b000001000, 3, 3));

        assert!(!in_bit_range(-3, 1, 0));
        assert!(!in_bit_range(-5, 2, 2));
        assert!(!in_bit_range(-5, 4, 3));
        assert!(in_bit_range(-1, 1, 0));
        assert!(in_bit_range(-4, 2, 0));
        assert!(in_bit_range(-5, 3, 2));
    }
}
