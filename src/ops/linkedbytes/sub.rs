use crate::{LBNum, LinkedByte};
use core::{ops, convert::TryInto};

impl LBNum {
    /// Decrements the byte at the specified index and returns the type of result.
    pub(crate) fn decrement_at_index(&mut self, index: usize) -> DecrementResult {
        if self.0.is_empty() {return DecrementResult::EndedWithBorrow;}
        if self.0.get(index).is_none() {return DecrementResult::NoSuchIndex;}
        for i in index.. {
            if let Some(refindex) = self.0.get_mut(i) {
                let (val, wrap) = refindex.sub_with_borrow(LinkedByte::from(1));
                *refindex = val;
                if !wrap {return DecrementResult::Ok(false);}
            } else {
                return DecrementResult::EndedWithBorrow;
            }
        }
        DecrementResult::Ok(true)
    }

    /// Performs checked subtraction. Returns `None` if the result underflowed 0, or the result wrapped in `Some` otherwise.
    #[inline(always)]
    pub fn checked_sub(mut self, rhs: &Self) -> Option<Self> {
        unsafe{ if !self.checked_sub_assign(rhs) {Some(self)} else {None} }
    }
    /// Performs checked subtraction, returning `true` if overflow occurred.
    ///
    /// # Safety
    /// If `true` is returned, the value of `self` is undefined.
    pub(crate) unsafe fn checked_sub_assign(&mut self, rhs: &Self) -> bool {
        if rhs.0.inner().is_empty() {return true;}
        if self.0.len() < rhs.0.len() {
            self.convert_last_to_linked();
            self.0.inner_mut().resize(rhs.0.len(), LinkedByte::ZERO_LINK);
            self.ensure_last_is_end();
        }
        for (i, other) in (0..self.0.len()).zip(rhs.0.iter_le()) {
            let this = &mut self.0.inner_mut()[i];
            let (val, wrapped) = this.sub_with_borrow(other);
            *this = val;
            if wrapped {
                if i == self.0.len() - 1 {
                    // If we're right at the end, we screwed up.
                    return true;
                } else {
                    // If not, decrement the next byte.
                    match self.decrement_at_index(i + 1) {
                        DecrementResult::Ok(_) => {},
                        DecrementResult::EndedWithBorrow | DecrementResult::NoSuchIndex => {return false}
                    }
                }
            } else {
                self.zero_fold();
                return false;
            }
        }
        self.zero_fold();
        false
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[must_use]
pub(crate) enum DecrementResult {
    /// The decrement-at-index operation was successful. The boolean parameter then describes whether wrapping occurred or not.
    Ok(bool),
    /// Index out of bounds.
    NoSuchIndex,
    /// The operation ended with a borrow, i.e. the number was too small to be decremented.
    ///
    /// In this case, the left operand is guaranteed to be zero.
    EndedWithBorrow
}

impl ops::Sub<&Self> for LBNum {
    type Output = Self;

    /// Subtracts an `LBNum` from `self`.
    ///
    /// # Panics
    /// Subtraction underflow is undefined for the Linked Bytes format, since it only specifies unsigned integers.
    #[inline(always)]
    fn sub(mut self, rhs: &Self) -> Self {
        self -= rhs;
        self
    }
}
impl ops::Sub<Self> for LBNum {
    type Output = Self;

    /// Subtracts an `LBNum` from `self`.
    ///
    /// # Panics
    /// Subtraction underflow is undefined for the Linked Bytes format, since it only specifies unsigned integers.
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        self - &rhs
    }
}
impl ops::SubAssign<&Self> for LBNum {
    /// Subtracts an `LBNum` from `self` in place.
    ///
    /// # Panics
    /// Subtraction underflow is undefined for the Linked Bytes format, since it only specifies unsigned integers.
    #[inline(always)]
    fn sub_assign(&mut self, rhs: &Self) {
        unsafe {
            if !self.checked_sub_assign(rhs) {
                panic!("BigBit integer underflow");
            }
        }
    }
}
impl ops::SubAssign<Self> for LBNum {
    /// Subtracts an `LBNum` from `self` in place.
    ///
    /// # Panics
    /// Subtraction underflow is undefined for the Linked Bytes format, since it only specifies unsigned integers.
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        *self -= &rhs;
    }
}

pub(crate) trait CheckedSub<'a, Rhs = &'a Self>: CheckedSubAssign<'a, Rhs> + Sized {
    /// Performs checked subtraction. Returns `None` if the result underflowed 0, or the result wrapped in `Some` otherwise.
    #[inline(always)]
    fn checked_sub(mut self, rhs: Rhs) -> Option<Self> {
        if unsafe {CheckedSubAssign::checked_sub_assign(&mut self, rhs)} {Some(self)} else {None}
    }
}
pub(crate) trait CheckedSubAssign<'a, Rhs = &'a Self> {
    unsafe fn checked_sub_assign(&mut self, rhs: Rhs) -> bool;
}

macro_rules! impl_sub_with_primitive {
    ($ty:ident) => {
        impl CheckedSub<'_, $ty> for LBNum {}
        impl CheckedSubAssign<'_, $ty> for LBNum {
            /// Performs checked subtraction, returning `true` if overflow occurred.
            ///
            /// # Safety
            /// If `true` is returned, the value of `self` is undefined.
            unsafe fn checked_sub_assign(&mut self, rhs: $ty) -> bool {
                if self.0.inner().get(0).is_none() {return true;} // Cannot subtract from 0.
                let div_by = LinkedByte::MAX + 1;
                let (rep, rem) = (rhs / div_by as $ty, rhs % div_by as $ty);
                for _ in 0..rep {
                    let (val, wrapped) = self.0.inner_mut()[0].sub_with_borrow(LinkedByte::from(127));
                    if wrapped {
                        match self.decrement_at_index(1) {
                            DecrementResult::Ok(_) => {},
                            DecrementResult::EndedWithBorrow | DecrementResult::NoSuchIndex => {return true;}
                        }
                    }
                    *self.0.get_mut(0).unwrap_or_else(||{core::hint::unreachable_unchecked()}) = val;
                }
                let rem: u8 = rem.try_into().unwrap();
                let (val, wrapped) = self.0.inner_mut()[0].sub_with_borrow(LinkedByte::from(rem));
                if wrapped {
                    match self.decrement_at_index(1) {
                        DecrementResult::Ok(_) => {},
                        DecrementResult::EndedWithBorrow | DecrementResult::NoSuchIndex => {return true;}
                    }
                }
                *self.0.get_mut(0).unwrap_or_else(||{core::hint::unreachable_unchecked()}) = val;
                self.zero_fold();
                false
            }
        }

        impl core::ops::Sub<$ty> for LBNum {
            type Output = Self;
            /// Subtracts `rhs` from `self`.
            ///
            /// # Panics
            /// Subtraction underflow is undefined for the Linked Bytes format, since it only specifies unsigned integers.
            #[inline(always)]
            fn sub(mut self, rhs: $ty) -> Self {
                self -= rhs;
                self
            }
        }
        /// Subtracts `rhs` from `self` in place.
        ///
        /// # Panics
        /// Subtraction underflow is undefined for the Linked Bytes format, since it only specifies unsigned integers.
        impl core::ops::SubAssign<$ty> for LBNum {
            fn sub_assign(&mut self, rhs: $ty) {
                if unsafe {CheckedSubAssign::checked_sub_assign(self, rhs)} {
                    panic!("BigBit integer underflow");
                }
            }
        }
    };
}

impl_sub_with_primitive!(u8   );
impl_sub_with_primitive!(u16  );
impl_sub_with_primitive!(u32  );
impl_sub_with_primitive!(u64  );
impl_sub_with_primitive!(u128 );
impl_sub_with_primitive!(usize);