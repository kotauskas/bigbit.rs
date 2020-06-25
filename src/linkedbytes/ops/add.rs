#![cfg_attr(feature = "clippy", allow(clippy::use_self))]

use crate::{
    linkedbytes::{LBNum, LBNumRef, LinkedByte},
    AddAssignAt,
};
use core::{
    ops::{Add, AddAssign},
    hint,
    convert::TryInto,
};

impl LBNum {
    /// Increments the byte at the specified index and returns whether wrapping occurred, or `None` if such an index does not exist.
    pub(crate) fn increment_at_index(&mut self, index: usize) -> Option<bool> {
        if self.0.is_empty() {
            self.0.inner_mut().push(LinkedByte::from(1));
            return Some(false);
        }
        if self.0.get(index).is_none() {
            if self.0.get(index - 1).is_some() {
                self.0.inner_mut()[index - 1].make_linked();
                self.0.inner_mut().push(LinkedByte::ZERO_END);
            } else {return None;}
        }
        for i in index.. {
            if let Some(refindex) = self.0.get_mut(i) {
                let (val, wrap) = refindex.add_with_carry(LinkedByte::from(1));
                *refindex = val;
                if !wrap {return Some(false);}
            } else {
                let len = self.0.inner().len();
                self.0.inner_mut()[len - 1].make_linked();
                self.0.inner_mut().push(LinkedByte::from(1).into_end());
                return Some(true);
            }
        }
        Some(true)
    }
}

// Implementation checklist:
// | lhs  |  rhs | LBNumRef  | reference | value | coreint   |
// | LBNumRef    | yes ~     | yes ~     | yes ~ | no        |
// | reference   | yes ~     | yes ~     | yes ~ | no        |
// | value       | yes ~     | yes ~     | yes ~ | yes ~     |
// | coreint     | no        | no        | yes ~ | N/A       |
// | value +=    | yes ~     | yes ~     | yes ~ | yes ~     |
// | coreint +=  | no        | no        | yes ~ | N/A       |
// | AddAssignAt | no        | no        | no    | yes ~     |
// AddAssign and AddAssignAt lhs is always value.

impl<'l, 'r> Add<LBNumRef<'r>> for LBNumRef<'l> {
    type Output = LBNum;
    #[inline(always)]
    fn add(self, rhs: LBNumRef<'r>) -> LBNum {
        Add::add(self.to_owned(), rhs)
    }
}
impl<'l, 'r> Add<&'r LBNum> for LBNumRef<'l> {
    type Output = LBNum;
    #[inline(always)]
    fn add(self, rhs: &'r LBNum) -> LBNum {
        Add::add(self.to_owned(), rhs.borrow())
    }
}
impl<'l> Add<LBNum> for LBNumRef<'l> {
    type Output = LBNum;
    #[inline(always)]
    fn add(self, rhs: LBNum) -> LBNum {
        Add::add(rhs, self)
    }
}

impl<'l, 'r> Add<LBNumRef<'r>> for &'l LBNum {
    type Output = LBNum;
    #[inline(always)]
    fn add(self, rhs: LBNumRef<'r>) -> LBNum {
        Add::add(self.clone(), rhs)
    }
}
impl<'l, 'r> Add<&'r LBNum> for &'l LBNum {
    type Output = LBNum;
    #[inline(always)]
    fn add(self, rhs: &'r LBNum) -> LBNum {
        Add::add(self.clone(), rhs.borrow())
    }
}
impl<'l> Add<LBNum> for &'l LBNum {
    type Output = LBNum;
    #[inline(always)]
    fn add(self, rhs: LBNum) -> LBNum {
        rhs + self
    }
}

impl<'r> Add<LBNumRef<'r>> for LBNum {
    type Output = LBNum;
    #[inline]
    fn add(mut self, rhs: LBNumRef<'r>) -> LBNum {
        AddAssign::add_assign(&mut self, rhs);
        self
    }
}
impl<'r> Add<&'r Self> for LBNum {
    type Output = LBNum;
    #[inline(always)]
    fn add(mut self, rhs: &'r Self) -> LBNum {
        Add::add(self, rhs.borrow())
    }
}
impl Add<Self> for LBNum {
    type Output = LBNum;
    #[inline(always)]
    fn add(self, rhs: Self) -> LBNum {
        Add::add(self, rhs.borrow())
    }
}

impl AddAssign<LBNumRef<'_>> for LBNum {
    #[inline]
    fn add_assign(&mut self, rhs: LBNumRef<'_>) {
        if rhs.is_empty() {return;}
        if self.0.len() < rhs.len() {
            self.0.inner_mut().resize(rhs.len(), LinkedByte::ZERO_LINK);
        }
        // Create a pair iterator. For every value of this, other is its corresponding value from rhs.
        for (i, other) in (0..self.0.len()).zip(rhs.iter_le()) {
            let this = &mut self.0.inner_mut()[i];
            let (val, wrapped) = this.add_with_carry(other);
            *this = val;
            if wrapped {
                if i == self.0.len() - 1 {
                    // If we're right at the end, just push a new element.
                    self.0.inner_mut().push(LinkedByte::from(1));
                } else {
                    // If not, increment the next byte.
                    self.increment_at_index(i + 1);
                }
            }
        }
        Self::fix_in_place(self.0.inner_mut());
    }
}
impl AddAssign<&LBNum> for LBNum {
    #[inline(always)]
    fn add_assign(&mut self, rhs: &LBNum) {
        AddAssign::add_assign(self, rhs.borrow())
    }
}
impl AddAssign<LBNum> for LBNum {
    #[inline(always)]
    fn add_assign(&mut self, rhs: LBNum) {
        AddAssign::add_assign(self, rhs.borrow())
    }
}

macro_rules! impl_add_with_primitive {
    ($($ty:ident)+) => ($(
        impl Add<$ty> for LBNum {
            type Output = LBNum;
            #[inline(always)]
            fn add(mut self, rhs: $ty) -> LBNum {self += rhs; self}
        }
        impl Add<LBNum> for $ty {
            type Output = LBNum;
            #[inline(always)]
            // Since addition is commutative, we can just switch the operands around and it's going to work automagically.
            fn add(self, rhs: LBNum) -> LBNum {rhs + self}
        }

        impl AddAssignAt<$ty> for LBNum {
            fn add_assign_at(&mut self, byte: usize, rhs: $ty) {
                if (self.0.inner().len() - 1) < byte {
                    self.0.inner_mut().resize(byte + 2, LinkedByte::ZERO_END);
                }
                let div_by = LinkedByte::MAX as u128 + 1;
                let (rep, rem) = (rhs as u128 / div_by, rhs as u128 % div_by);
                for _ in 0..rep {
                    let (val, wrapped) = self.0.inner_mut()[byte + 1].add_with_carry(LinkedByte::from(1));
                    if wrapped {self.increment_at_index(byte + 2);}
                    // The check at the beginning of the function lets us be sure that there is always a byte at that index.
                    *self.0.get_mut(byte + 1).unwrap_or_else(||{unsafe {hint::unreachable_unchecked()}}) = val;
                }
                let rem: u8 = rem.try_into().unwrap();
                let (val, wrapped) = self.0.inner_mut()[byte].add_with_carry(LinkedByte::from(rem));
                if wrapped {self.increment_at_index(byte + 1);}
                // Same justification as above.
                *self.0.get_mut(byte).unwrap_or_else(||{unsafe {hint::unreachable_unchecked()}}) = val;

                self.zero_fold();
                Self::fix_in_place(&mut self.0.inner_mut()[..]);
            }
        }

        impl AddAssign<$ty> for LBNum {
            #[inline(always)]
            fn add_assign(&mut self, rhs: $ty) {
                self.add_assign_at(0, rhs);
            }
        }
        impl AddAssign<LBNum> for $ty {
            #[inline]
            fn add_assign(&mut self, rhs: LBNum) {
                if let Ok(val) = TryInto::try_into(*self + rhs) {
                    *self = val;
                } else {
                    panic!("integer overflow while adding a BigBit number to a primitive integer");
                }
            }
        }
    )+)
}
impl_add_with_primitive!{
    u8 u16 u32 u64 u128 usize
}