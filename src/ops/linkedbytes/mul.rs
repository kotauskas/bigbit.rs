use crate::LBNum;
use core::ops;

impl ops::Mul for LBNum {
    type Output = Self;

    /// Multiplies `self` by another `LBNum`. **This will consume `rhs`**.
    #[inline(always)]
    fn mul(mut self, rhs: Self) -> Self {
        self *= rhs;
        self
    }
}
impl ops::MulAssign for LBNum {
    /// Multiplies `self` by another `LBNum` in place. **This will consume `rhs`**.
    #[inline]
    fn mul_assign(&mut self, mut rhs: Self) {
        let mut result = Self::ZERO;
        loop {
            if rhs == Self::ZERO {break;}
            result += self as &Self;
            rhs.decrement();
        }
        *self = result;
    }
}

macro_rules! impl_mul_by_primitive {
    ($ty:ident) => {
        impl ops::Mul<$ty> for LBNum {
            type Output = Self;

            #[inline(always)]
            fn mul(mut self, rhs: $ty) -> Self {
                self *= rhs;
                self
            }
        }
        impl ops::MulAssign<$ty> for LBNum {
            #[inline]
            fn mul_assign(&mut self, mut rhs: $ty) {
                let mut result = Self::ZERO;
                loop {
                    if rhs == 0 {break;}
                    result += self as &Self;
                    rhs -= 1;
                }
                *self = result;
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