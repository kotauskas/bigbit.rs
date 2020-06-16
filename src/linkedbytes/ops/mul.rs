#![cfg_attr(feature = "clippy", allow(clippy::use_self))] // Multiplication impl blocks do this intentionally.

use crate::LBNum;
use core::{
    ops,
    mem::swap,
};

// Reference by reference
impl ops::Mul<&LBNum> for &LBNum {
    type Output = LBNum;

    /// Multiplies `self` by another `LBNum`. **This will consume neither of the operands.**
    #[inline]
    fn mul(self, rhs: &LBNum) -> LBNum {
        let mut result = LBNum::ZERO;
        let (mut left, mut right) = (self, rhs);
        if right.0.len() > left.0.len() {
            // This swaps the borrows so that the left one always has more bytes
            swap(&mut left, &mut right);
        }
        for byte_on_left in left.iter_le() {
            for byte_on_right in right.iter_le() {
                let terms_multiplied
                    = (byte_on_left.into_int7() as u16) * (byte_on_right.into_int7() as u16);
                result += terms_multiplied;
            }
        }
        result
    }
}
// Value by reference
impl ops::Mul<&LBNum> for LBNum {
    type Output = LBNum;

    /// Multiplies `self` by another `LBNum`. **This will consume the lefthand operand, but not the righthand one. Borrow the lefthand operand to avoid such behavior.**
    #[inline(always)]
    fn mul(self, rhs: &LBNum) -> LBNum {
        ops::Mul::mul(&self, rhs)
    }
}
// Reference by value
impl ops::Mul<LBNum> for &LBNum {
    type Output = LBNum;

    /// Multiplies `self` by another `LBNum`. **This will consume the righthand operand, but not the lefthand one. Borrow the righthand operand to avoid such behavior.**
    #[inline(always)]
    fn mul(self, rhs: LBNum) -> LBNum {
        ops::Mul::mul(self, &rhs)
    }
}
// Value by value
impl ops::Mul<LBNum> for LBNum {
    type Output = LBNum;

    /// Multiplies `self` by another `LBNum`. **This will consume both operands, borrow them to avoid such behavior.**
    #[inline(always)]
    fn mul(self, rhs: Self) -> Self {
        ops::Mul::mul(&self, &rhs)
    }
}

// By reference
impl ops::MulAssign<&LBNum> for LBNum {
    /// Multiplies `self` by another `LBNum` in place. **This will not consume the righthand operand.**
    #[inline(always)]
    fn mul_assign(&mut self, rhs: &LBNum) {
        *self = ops::Mul::mul(self as &LBNum, rhs);
    }
}
// By value
impl ops::MulAssign<LBNum> for LBNum {
    /// Multiplies `self` by another `LBNum` in place. **This will consume the righthand operand, borrow it to avoid such behavior.**
    #[inline(always)]
    fn mul_assign(&mut self, rhs: LBNum) {
        *self = ops::Mul::mul(self as &LBNum, &rhs);
    }
}

macro_rules! impl_mul_by_primitive {
    ($ty:ident) => {
        // Reference by value
        impl ops::Mul<$ty> for &LBNum {
            type Output = LBNum;

            #[inline]
            fn mul(self, rhs: $ty) -> LBNum {
                let mut result = LBNum::ZERO;
                for byte_on_left in self.iter_le() {
                    let terms_multiplied
                        = ((byte_on_left.into_int7() as $ty) * rhs);
                    result += terms_multiplied;
                }
                result
            }
        }
        // Value by value
        impl ops::Mul<$ty> for LBNum {
            type Output = LBNum;

            #[inline(always)]
            fn mul(self, rhs: $ty) -> LBNum {
                ops::Mul::mul(&self, rhs)
            }
        }
        // By value
        impl ops::MulAssign<$ty> for LBNum {
            #[inline(always)]
            fn mul_assign(&mut self, rhs: $ty) {
                *self = ops::Mul::mul(self as &LBNum, rhs)
            }
        }
    };
}

impl_mul_by_primitive!(u8   );
impl_mul_by_primitive!(u16  );
impl_mul_by_primitive!(u32  );
impl_mul_by_primitive!(u64  );
impl_mul_by_primitive!(u128 );
impl_mul_by_primitive!(usize);