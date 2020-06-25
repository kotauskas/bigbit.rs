#![cfg_attr(feature = "clippy", allow(clippy::use_self))] // Multiplication impl blocks do this intentionally.

use crate::{
    linkedbytes::{LBNum, LBNumRef},
    AddAssignAt,
};
use core::{
    ops,
    mem::swap,
};

// Implementation checklist:
// | lhs | rhs | LBNumRef | reference | value | coreint |
// | LBNumRef  | primary  | yes       | yes   | yes     |
// | reference | yes      | yes       | yes   | yes     |
// | value     | yes      | yes       | yes   | yes     |
// | coreint   | yes      | yes       | yes   | N/A     |
// | value *=  | yes      | yes       | yes   | yes     |
// MulAssign's lhs is always value.
// Should be grouped in blocks of 3 by left operand.

// LBNumRef by LBNumRef
impl<'l, 'r> ops::Mul<LBNumRef<'r>> for LBNumRef<'l> {
    type Output = LBNum;
    /// Multiplies an `LBNumRef` **by another `LBNumRef`**.
    #[inline]
    fn mul(self, rhs: LBNumRef<'r>) -> LBNum {
        let mut result = LBNum::ZERO;
        let (mut left, mut right) = (self, rhs);
        if right.0.len() > left.0.len() {
            // This swaps the borrows so that the left one always has more bytes
            swap(&mut left, &mut right);
        }
        for (byte_on_left, byte_on_left_index) in left.iter_le().zip(0_usize..) {
            for byte_on_right in right.iter_le() {
                let terms_multiplied
                    = (byte_on_left.into_int7() as u16) * (byte_on_right.into_int7() as u16);
                result.add_assign_at(byte_on_left_index, terms_multiplied);
            }
        }
        result
    }
}
// LBNumRef by reference
impl<'l, 'r> ops::Mul<&'r LBNum> for LBNumRef<'l> {
    type Output = LBNum;
    // Multiplies an `LBNumRef` **by a reference to `LBNum`**.
    #[inline(always)]
    fn mul(self, rhs: &'r LBNum) -> LBNum {
        ops::Mul::mul(self, rhs.borrow())
    }
}
// LBNumRef by value
impl<'l> ops::Mul<LBNum> for LBNumRef<'l> {
    type Output = LBNum;
    // Multiplies an `LBNumRef` **by an `LBNum`, consuming it.** Borrow the righthand operand (via normal borrow or `.borrow()`) to avoid such behavior.
    #[inline(always)]
    fn mul(self, rhs: LBNum) -> LBNum {
        ops::Mul::mul(self, rhs.borrow())
    }
}

// Reference by LBNumRef
impl<'l, 'r> ops::Mul<LBNumRef<'r>> for &LBNum {
    type Output = LBNum;
    // Multiplies an `LBNum` *reference* **by an `LBNumRef`**.
    #[inline(always)]
    fn mul(self, rhs: LBNumRef<'r>) -> LBNum {
        ops::Mul::mul(self.borrow(), rhs)
    }
}
// Reference by reference
impl<'l, 'r> ops::Mul<&'r LBNum> for &'l LBNum {
    type Output = LBNum;
    // Multiplies an `LBNum` *reference* **by another `LBNum` reference.**
    #[inline(always)]
    fn mul(self, rhs: &'r LBNum) -> LBNum {
        ops::Mul::mul(self.borrow(), rhs.borrow())
    }
}
// Reference by value
impl<'l> ops::Mul<LBNum> for &'l LBNum {
    type Output = LBNum;
    /// Multiplies an `LBNum` *reference* **by an `LBNum`, consuming it.** Borrow the righthand operand (via normal borrow or `.borrow()`) to avoid such behavior.
    #[inline(always)]
    fn mul(self, rhs: LBNum) -> LBNum {
        ops::Mul::mul(self.borrow(), rhs.borrow())
    }
}

// Value by LBNumRef
impl<'r> ops::Mul<LBNumRef<'r>> for LBNum {
    type Output = LBNum;
    // Multiplies an `LBNum` **by an `LBNumRef`,** consuming the **lefthand** operand. Borrow it (via normal borrow or `.borrow()`) to avoid such behavior.
    #[inline(always)]
    fn mul(self, rhs: LBNumRef<'r>) -> LBNum {
        ops::Mul::mul(self.borrow(), rhs)
    }
}
// Value by reference
impl<'r> ops::Mul<&'r LBNum> for LBNum {
    type Output = LBNum;
    /// Multiplies an `LBNum` **by an `LBNum` reference.**, consuming the **lefthand** operand. Borrow it (via normal borrow or `.borrow()`) to avoid such behavior.
    #[inline(always)]
    fn mul(self, rhs: &'r LBNum) -> LBNum {
        ops::Mul::mul(self.borrow(), rhs.borrow())
    }
}
// Value by value
impl ops::Mul<LBNum> for LBNum {
    type Output = LBNum;
    /// Multiplies an `LBNum` **by another `LBNum`, consuming both operands.** Borrow them (via normal borrow or `.borrow()`) to avoid such behavior.
    #[inline(always)]
    fn mul(self, rhs: Self) -> Self {
        ops::Mul::mul(self.borrow(), rhs.borrow())
    }
}

// By LBNumRef
impl<'r> ops::MulAssign<LBNumRef<'r>> for LBNum {
    // Multiplies **by an `LBNumRef`** in place.
    #[inline(always)]
    fn mul_assign(&mut self, rhs: LBNumRef<'r>) {
        *self = ops::Mul::mul(self.borrow(), rhs)
    }
}
// By reference
impl ops::MulAssign<&LBNum> for LBNum {
    /// Multiplies **by an `LBNum` reference** in place.
    #[inline(always)]
    fn mul_assign(&mut self, rhs: &LBNum) {
        *self = ops::Mul::mul(self.borrow(), rhs.borrow());
    }
}
// By value
impl ops::MulAssign<LBNum> for LBNum {
    /// Multiplies **by another `LBNum`** in place, **consuming it.** Borrow the righthand operand (via normal borrow or `.borrow()`) to avoid such behavior.
    #[inline(always)]
    fn mul_assign(&mut self, rhs: LBNum) {
        *self = ops::Mul::mul(self.borrow(), rhs.borrow());
    }
}

macro_rules! impl_mul_by_primitive {
    ($ty:ident) => {
        // LBNumRef by int
        impl<'l> ops::Mul<$ty> for LBNumRef<'l> {
            type Output = LBNum;
            #[doc = "Multiplies an `LBNumRef` **by a scalar integer.**"]
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
        // Reference by int
        impl<'l> ops::Mul<$ty> for &'l LBNum {
            type Output = LBNum;
            #[doc = "Multiplies an `LBNum` *reference* **by a scalar integer.**"]
            #[inline(always)]
            fn mul(self, rhs: $ty) -> LBNum {
                ops::Mul::mul(self.borrow(), rhs)
            }
        }
        // Value by int
        impl ops::Mul<$ty> for LBNum {
            type Output = LBNum;
            #[doc = "Multiplies an `LBNum` **by a scalar integer,** consuming the **lefthand** operand. Borrow it (via normal borrow or `.borrow()`) to avoid such behavior"]
            #[inline(always)]
            fn mul(self, rhs: $ty) -> LBNum {
                ops::Mul::mul(self.borrow(), rhs)
            }
        }

        // Int by LBNumRef
        impl<'r> ops::Mul<LBNumRef<'r>> for $ty {
            type Output = LBNum;
            #[doc = "Multiplies a scalar integer **by an `LBNumRef`.**"]
            #[inline(always)]
            fn mul(self, rhs: LBNumRef<'r>) -> LBNum {
                ops::Mul::mul(rhs, self)
            }
        }
        // Int by reference
        impl<'r> ops::Mul<&'r LBNum> for $ty {
            type Output = LBNum;
            #[doc = "Multiplies a scalar integer **by an `LBNum` reference.**"]
            #[inline(always)]
            fn mul(self, rhs: &'r LBNum) -> LBNum {
                ops::Mul::mul(rhs.borrow(), self)
            }
        }
        // Int by value
        impl ops::Mul<LBNum> for $ty {
            type Output = LBNum;
            #[doc = "Multiplies a scalar integer **by an `LBNum`, consuming it.** Borrow the righthand operand (via normal borrow or `.borrow()`) to avoid such behavior."]
            #[inline(always)]
            fn mul(self, rhs: LBNum) -> LBNum {
                ops::Mul::mul(rhs.borrow(), self)
            }
        }

        // By int
        impl ops::MulAssign<$ty> for LBNum {
            #[inline(always)]
            #[doc = "Multiplies an `LBNum` **by a scalar integer** in place."]
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