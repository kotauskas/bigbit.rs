use crate::{LBNum, LinkedByte};
use core::{
    cmp::{self, Ordering},
    ops
};

impl LBNum {
    /// Increments the byte at the specified index and returns whether wrapping occurred, or `None` if such an index does not exist.
    pub(crate) fn increment_at_index(&mut self, index: usize) -> Option<bool> {
        if self.0.is_empty() {
            self.0.inner_mut().push(LinkedByte::from(1));
            return Some(false);
        }
        self.0.get(index)?;
        for i in index.. {
            if let Some(refindex) = self.0.get_mut(i) {
                let (val, wrap) = refindex.add_with_carry(LinkedByte::from(1));
                *refindex = val;
                if !wrap {return Some(false);}
            } else {
                self.0.inner_mut().push(LinkedByte::from(1));
                return Some(true);
            }
        }
        Some(true)
    }
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
    /// Performs checked subtraction.
    ///
    /// # Safety
    /// If `false` is returned, the result is undefined.
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
                    return false;
                } else {
                    // If not, decrement the next byte.
                    match self.decrement_at_index(i + 1) {
                        DecrementResult::Ok(_) => {},
                        DecrementResult::EndedWithBorrow | DecrementResult::NoSuchIndex => {return false}
                    }
                }
            } else {
                return true;
            }
        }
        true
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

impl ops::Add<&Self> for LBNum {
    type Output = Self;

    #[inline(always)]
    #[must_use]
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
impl ops::Sub<&Self> for LBNum {
    type Output = Self;

    #[inline(always)]
    #[must_use]
    fn sub(mut self, rhs: &Self) -> Self {
        self -= rhs;
        self
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
impl cmp::PartialEq for LBNum {
    fn eq(&self, rhs: &Self) -> bool {
        if self == rhs {return true;}
        for (this, other) in self.iter_le().zip(rhs.iter_le()) {
            if this != other {
                return false;
            }
        }
        true
    }
}
impl cmp::Eq for LBNum {}
impl cmp::PartialOrd for LBNum {
    #[inline(always)]
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}
impl cmp::Ord for LBNum {
    fn cmp(&self, rhs: &Self) -> Ordering {
        match self.0.len().cmp(&rhs.0.len()) {
            Ordering::Greater => Ordering::Greater,
            Ordering::Less => Ordering::Less,
            Ordering::Equal => {
                for (this, other) in self.iter_be().zip(rhs.iter_be()) {
                    match this.cmp(&other) {
                        Ordering::Less => {return Ordering::Less},
                        Ordering::Greater => {return Ordering::Greater},
                        Ordering::Equal => {} // Do nothing in this case, search for the next one.
                    }
                }
                Ordering::Equal
            }
        }
    }
}