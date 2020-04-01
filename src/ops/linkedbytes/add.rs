use crate::{LBNum, LinkedByte};
use core::{ops, convert::TryInto};

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

impl ops::Add<&Self> for LBNum {
    type Output = Self;

    #[inline(always)]
    fn add(mut self, rhs: &Self) -> Self {
        self += rhs;
        self
    }
}
impl ops::AddAssign<&Self> for LBNum {
    fn add_assign(&mut self, rhs: &Self) {
        if rhs.0.inner().is_empty() {return;}
        if self.0.len() < rhs.0.len() {
            self.convert_last_to_linked();
            self.0.inner_mut().resize(rhs.0.len(), LinkedByte::ZERO_LINK);
            self.ensure_last_is_end();
        }
        // Create a pair iterator. For every value of this, other is its corresponding value from rhs.
        for (i, other) in (0..self.0.len()).zip(rhs.0.iter_le()) {
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
    }
}

macro_rules! impl_add_with_primitive {
    ($ty:ident) => {
        impl core::ops::Add<$ty> for LBNum {
            type Output = Self;
            #[inline(always)]
            fn add(mut self, rhs: $ty) -> Self {self += rhs; self}
        }
        impl core::ops::AddAssign<$ty> for LBNum {
            fn add_assign(&mut self, rhs: $ty) {
                if self.0.inner().get(0).is_none() {
                    self.0.inner_mut().push(LinkedByte::ZERO_END);
                }
                let div_by = LinkedByte::MAX + 1;
                let (rep, rem) = (rhs / div_by as $ty, rhs % div_by as $ty);
                for _ in 0..rep {
                    let (val, wrapped) = self.0.inner_mut()[0].add_with_carry(LinkedByte::from(127));
                    if wrapped {self.increment_at_index(1);}
                    *self.0.get_mut(0).unwrap_or_else(||{unsafe{core::hint::unreachable_unchecked()}}) = val;
                }
                let rem: u8 = rem.try_into().unwrap();
                let (val, wrapped) = self.0.inner_mut()[0].add_with_carry(LinkedByte::from(rem));
                if wrapped {self.increment_at_index(1);}
                *self.0.get_mut(0).unwrap_or_else(||{unsafe{core::hint::unreachable_unchecked()}}) = val;
            }
        }
    };
}
impl_add_with_primitive!(u8   );
impl_add_with_primitive!(u16  );
impl_add_with_primitive!(u32  );
impl_add_with_primitive!(u64  );
impl_add_with_primitive!(u128 );
impl_add_with_primitive!(usize);