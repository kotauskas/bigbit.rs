#![cfg_attr(feature = "clippy", allow(clippy::use_self))]

use crate::{
    linkedbytes::{LBNum, LBNumRef},
    DivRem, DivRemAssign,
};
use core::{
    ops::{Div, DivAssign, Rem, RemAssign},
    mem,
};

impl DivRem<&Self> for LBNum {
    type Quotient = Self; type Remainder = Self;

    /// Performs combined integer division and remainder calculation.
    ///
    /// # Panics
    /// Dividing by 0 triggers an immediate panic.
    #[inline(always)]
    fn div_rem(mut self, rhs: &Self) -> (Self, Self) {
        let remainder = self.div_rem_assign(rhs);
        (self, remainder)
    }
}
impl DivRem<Self> for LBNum {
    type Quotient = Self; type Remainder = Self;

    /// Performs combined integer division and remainder calculation.
    ///
    /// # Panics
    /// Dividing by 0 triggers an immediate panic.
    #[inline(always)]
    fn div_rem(self, rhs: Self) -> (Self, Self) {
        self.div_rem(&rhs)
    }
}
impl DivRemAssign<&Self> for LBNum {
    type Remainder = Self;

    /// Performs in-place integer division combined with returning the remainder.
    ///
    /// # Panics
    /// Dividing by 0 triggers an immediate panic.
    #[inline]
    fn div_rem_assign(&mut self, rhs: &Self) -> Self {
        assert!(rhs > &0_u8);
        let mut quotient = Self::ZERO;
        loop {
            if (self as &Self) < rhs {break;}
            unsafe {self.checked_sub_assign(rhs.into());}
            quotient.increment();
        }
        core::mem::replace(self, quotient) // This moves the remainder out of self, moves the quotient and tail-returns it to the callee.
                                           // While this kind of call might indeed be unintuitive, reading the core::mem::replace docs is
                                           // all you need to do to understand this just fine.
    }
}
impl DivRemAssign<Self> for LBNum {
    type Remainder = Self;

    /// Performs in-place integer division combined with returning the remainder.
    ///
    /// # Panics
    /// Dividing by 0 triggers an immediate panic.
    #[inline(always)]
    fn div_rem_assign(&mut self, rhs: Self) -> Self {
        self.div_rem_assign(&rhs)
    }
}
impl DivRemAssign<LBNumRef<'_>> for LBNum {
    type Remainder = Self;

    /// Performs in-place integer division combined with returning the remainder.
    ///
    /// # Panics
    /// Dividing by 0 triggers an immediate panic.
    fn div_rem_assign(&mut self, rhs: LBNumRef<'_>) -> Self {
        assert!(rhs > 0_u8);
        let mut quotient = Self::ZERO;
        loop {
            if *self < rhs {break;}
            unsafe {self.checked_sub_assign(rhs);}
            quotient.increment();
        }
        core::mem::replace(self, quotient) // This moves the remainder out of self, moves the quotient and tail-returns it to the callee.
                                           // While this kind of call might indeed be unintuitive, reading the core::mem::replace docs is
                                           // all you need to do to understand this just fine.
    }
}
impl Div<&Self> for LBNum {
    type Output = Self;

    /// Performs integer division.
    ///
    /// # Panics
    /// Dividing by 0 triggers an immediate panic.
    #[inline(always)]
    fn div(self, rhs: &Self) -> Self {
        self.div_rem(rhs).0
    }
}
impl Div<Self> for LBNum {
    type Output = Self;

    /// Performs integer division.
    ///
    /// # Panics
    /// Dividing by 0 triggers an immediate panic.
    #[inline(always)]
    fn div(self, rhs: Self) -> Self {
        self / &rhs
    }
}
impl DivAssign<&Self> for LBNum {
    /// Performs integer division in place.
    ///
    /// # Panics
    /// Dividing by 0 triggers an immediate panic.
    #[inline(always)]
    fn div_assign(&mut self, rhs: &Self) {
        self.div_rem_assign(rhs);
    }
}
impl DivAssign<Self> for LBNum {
    /// Performs integer division in place.
    ///
    /// # Panics
    /// Dividing by 0 triggers an immediate panic.
    #[inline(always)]
    fn div_assign(&mut self, rhs: Self) {
        *self /= &rhs;
    }
}
impl Rem<&Self> for LBNum {
    type Output = Self;

    /// Performs integer modulo.
    ///
    /// # Panics
    /// Dividing by 0 triggers an immediate panic.
    #[inline(always)]
    fn rem(self, rhs: &Self) -> Self {
        self.div_rem(rhs).1
    }
}
impl Rem<Self> for LBNum {
    type Output = Self;
    /// Performs integer modulo.
    ///
    /// # Panics
    /// Dividing by 0 triggers an immediate panic.
    #[inline(always)]
    fn rem(self, rhs: Self) -> Self {
        self % &rhs
    }
}
impl RemAssign<&Self> for LBNum {
    /// Performs integer modulo in place.
    ///
    /// # Panics
    /// Dividing by 0 triggers an immediate panic.
    #[inline(always)]
    fn rem_assign(&mut self, rhs: &Self) {
        let remainder = self.div_rem_assign(rhs);
        *self = remainder;
    }
}
impl RemAssign<Self> for LBNum {
    /// Performs integer modulo in place.
    ///
    /// # Panics
    /// Dividing by 0 triggers an immediate panic.
    #[inline(always)]
    fn rem_assign(&mut self, rhs: Self) {
        *self %= &rhs;
    }
}

macro_rules! impl_div_by_primitive {
    ($($ty:ident)+) => ($(
        impl DivRem<$ty> for LBNum {
            type Quotient = Self;
            /// The remainder type.
            ///
            /// The reason why this is `Self` instead of the type of the divisor is that the remainder as available when the division is finished is still of type `LBNum`: it's never converted to the divisor type. As a result, the remainder is returned as-is to avoid situations when the remainder is required to be an `LBNum` yet has been converted to the divisor type, which would require converting it back into `LBNum`, which would require another allocation *and* performing the conversion process itself and would also waste the previous buffer.
            type Remainder = Self;

            #[inline(always)]
            fn div_rem(mut self, rhs: $ty) -> (Self, Self) {
                let remainder = self.div_rem_assign(rhs);
                (self, remainder)
            }
        }
        impl DivRemAssign<$ty> for LBNum {
            /// The remainder type.
            ///
            /// The reason why this is `Self` instead of the type of the divisor is that the remainder as available when the division is finished is still of type `LBNum`: it's never converted to the divisor type. As a result, the remainder is returned as-is to avoid situations when the remainder is required to be an `LBNum` yet has been converted to the divisor type, which would require converting it back into `LBNum`, which would require another allocation *and* performing the conversion process itself and would also waste the previous buffer.
            type Remainder = Self;

            #[inline]
            fn div_rem_assign(&mut self, rhs: $ty) -> Self {
                assert!(rhs > 0);
                let mut quotient = Self::ZERO;
                loop {
                    if (self as &Self) < &rhs {break;}
                    unsafe {self.checked_sub_assign(LBNum::from(rhs).borrow());}
                    quotient.increment();
                }
                mem::replace(self, quotient)
            }
        }


        impl Div<$ty> for LBNum {
            type Output = Self;

            #[inline(always)]
            fn div(mut self, rhs: $ty) -> Self {
                self /= rhs;
                self
            }
        }
        impl DivAssign<$ty> for LBNum {
            #[inline]
            fn div_assign(&mut self, rhs: $ty) {
                assert!(rhs > 0);
                let mut result = Self::ZERO;
                loop {
                    if (self as &Self) < &rhs {break;}
                    unsafe {self.checked_sub_assign(LBNum::from(rhs).borrow());}
                    result.increment();
                }
                *self = result;
            }
        }

        impl Rem<$ty> for LBNum {
            /// The remainder type.
            ///
            /// The reason why this is `Self` instead of the type of the divisor is that the remainder as available when the division is finished is still of type `LBNum`: it's never converted to the divisor type. As a result, the remainder is returned as-is to avoid situations when the remainder is required to be an `LBNum` yet has been converted to the divisor type, which would require converting it back into `LBNum`, which would require another allocation *and* performing the conversion process itself and would also waste the previous buffer.
            type Output = Self;
            #[inline(always)]
            fn rem(self, rhs: $ty) -> Self {
                self.div_rem(rhs).1
            }
        }
        impl RemAssign<$ty> for LBNum {
            #[inline(always)]
            fn rem_assign(&mut self, rhs: $ty) {
                let remainder = self.div_rem_assign(rhs);
                *self = remainder
            }
        }
    )+)
}

impl_div_by_primitive! {
    u8 u16 u32 u64 u128 usize
}