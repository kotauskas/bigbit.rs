//! The Linked Bytes format, capable of storing arbitrarily large non-negative integers, also used to efficently store Unicode strings.
//!
//! If you only want non-negative integers, you should stick to this format. Otherwise, use either Head Byte or Extended Head Byte.

use alloc::vec::Vec;

/// A number in the Linked Bytes format, capable of storing arbitrarily large non-negative integers.
#[derive(Clone)]
pub struct LBNum(Vec<LinkedByte>);

impl LBNum {
    /// The zero value.
    ///
    /// This does not allocate memory.
    pub const ZERO: Self = Self(Vec::new());
    /// The amount of bytes used in the number.
    #[inline(always)]
    #[must_use]
    pub fn num_bytes(&self) -> usize {
        self.0.len()
    }
    /// Increments the value.
    #[inline(always)]
    pub fn increment(&mut self) {
        self.increment_at_index(0);
    }

    /// Ensures that the last element is an endpoint.
    pub(crate) fn ensure_last_is_end(&mut self) {
        if let Some(last) = self.0.last_mut() {
            *last = last.into_end()
        } else {}
    }
    /// Converts the last element to a linked byte, for adding together two `LBNum`s.
    pub(crate) fn convert_last_to_linked(&mut self) {
        if let Some(last) = self.0.last_mut() {
            *last = last.into_linked()
        } else {}
    }
    /// Increments the byte at the specified index and returns whether wrapping occurred, or `None` if such an index does not exist.
    pub(crate) fn increment_at_index(&mut self, index: usize) -> Option<bool> {
        if self.0.is_empty() {
            self.0.push(LinkedByte::from(1));
            return Some(false);
        }
        self.0.get(index)?;
        for i in index..self.0.len() {
            if let Some(refindex) = self.0.get_mut(i) {
                let (val, wrap) = refindex.add_with_carry(LinkedByte::from(1));
                *refindex = val;
                if !wrap {return Some(false);}
            } else {
                self.0.push(LinkedByte::from(1));
                return Some(true);
            }
        }
        Some(true)
    }
}
impl core::ops::Add<&Self> for LBNum {
    type Output = Self;
    #[inline(always)]
    #[must_use]
    fn add(mut self, rhs: &Self) -> Self {
        self += rhs;
        self
    }
}
impl core::ops::AddAssign<&Self> for LBNum {
    fn add_assign(&mut self, rhs: &Self) {
        if rhs.0.is_empty() {return;}
        if self.0.len() < rhs.0.len() {
            self.convert_last_to_linked();
            self.0.resize(rhs.0.len(), LinkedByte::ZERO_LINK);
            self.ensure_last_is_end();
        }
        // Create a pair iterator. For every value of this, other is its corresponding value from rhs.
        for (i, other) in (0..self.0.len()).zip(rhs.0.iter()) {
            let this = &mut self.0[i];
            let (val, wrapped) = this.add_with_carry(*other);
            *this = val;
            if wrapped {
                if i == self.0.len() - 1 {
                    // If we're right at the end, just push a new element.
                    self.0.push(LinkedByte::from(1));
                } else {
                    // If not, increment the next byte.
                    self.increment_at_index(i + 1);
                }
            }
        }
    }
}

/// An element in a series of Linked Bytes.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct LinkedByte (u8);

impl LinkedByte {
    /// The link bit mask.
    ///
    /// Use [`is_linked`][0] to easily check for this.
    ///
    /// [0]: #method.is_linked "is_linked — returns true if the byte is linked to a following byte, false otherwise"
    pub const LINK_MASK: u8 = 0b1_0000000;
    /// The value bit mask.
    ///
    /// Use [`value`][0] to easily check for this.
    ///
    /// [0]: #method.value "value — returns true if the byte is linked to a following byte, false otherwise"
    pub const VALUE_MASK: u8 = !Self::LINK_MASK;

    /// The zero value as an endpoint (no follow-up bytes expected).
    pub const ZERO_END: Self = Self(0);
    /// The zero value as a linked byte (one or more follow-up bytes expected).
    pub const ZERO_LINK: Self = Self(Self::LINK_MASK);

    /// Returns `true` if the byte is linked to a following byte, `false` otherwise.
    #[inline(always)]
    #[must_use]
    pub fn is_linked(self) -> bool {
        (self.0 & Self::LINK_MASK) != 0
    }
    /// Returns the value of the linked byte, in the range from 0 to 127, inclusively.
    ///
    /// The main use for this is performing arithmetic with linked bytes.
    #[inline(always)]
    #[must_use]
    pub fn value(self) -> u8 {
        self.0 & Self::VALUE_MASK
    }
    /// Sets the link bit to `true` (linked state).
    #[inline(always)]
    #[must_use]
    pub fn into_linked(self) -> Self {
        Self(self.0 | Self::ZERO_LINK.0)
    }
    /// Sets the link bit to `false` (endpoint state).
    #[inline(always)]
    #[must_use]
    pub fn into_end(self) -> Self {
        Self(self.0 & Self::VALUE_MASK)
    }
    /// Performs checked addition. `None` is returned if the result overflows the limit of 127.
    #[inline]
    #[must_use]
    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        let (lhs_end, rhs_end) = (self.into_end().0, rhs.into_end().0);
        if let Some(nonwrapping) = lhs_end.checked_add(rhs_end) {
            if Self(nonwrapping).is_linked() {None} else {
                let mut result = Self(nonwrapping);
                if self.is_linked() {result = result.into_linked()}
                Some(result)
            }
        } else {None}
    }
    /// Performs checked wrapping addition. Unlike [`checked_add`][0], this method returns a tuple, in which the first value is the result, which wraps over if the result overflows the limit of 127, and the second value is whether the overflow actually occurred.
    ///
    /// [0]: #method.checked_add "checked_add — perform checked addition"
    #[inline]
    #[must_use]
    pub fn add_with_carry(self, rhs: Self) -> (Self, bool) {
        if let Some(nonwrapping) = self.checked_add(rhs) {
            (nonwrapping, false)
        } else {
            (Self(self.0.wrapping_add(rhs.0)), true)
        }
    }
    /// Performs checked subtraction. `None` is returned if the result underflows the limit of 0.
    #[inline]
    #[must_use]
    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        let (lhs_end, rhs_end) = (self.into_end(), rhs.into_end());
        if let Some(nonwrapping) = lhs_end.0.checked_sub(rhs_end.0) {
            let mut result = Self(nonwrapping);
            if self.is_linked() {result = result.into_linked()}
            Some(result)
        } else {None}
    }
    /// Performs checked wrapping subtraction. Unlike [`checked_sub`][0], this method returns a tuple, in which the first value is the result, which wraps over if the result underflows the limit of 0, and the second value is whether the overflow actually occurred.
    ///
    /// [0]: #method.checked_sub "checked_sub — perform checked subtraction"
    pub fn sub_with_borrow(self, rhs: Self) -> (Self, bool) {
        if let Some(nonwrapping) = self.checked_sub(rhs) {
            (nonwrapping, false)
        } else {
            (Self(self.0.wrapping_sub(rhs.0)), true)
        }
    }

    // Consumes the value and returns the inner byte.
    #[inline(always)]
    #[must_use]
    pub fn into_inner(self) -> u8 {
        self.0
    }
}
impl From<u8> for LinkedByte {
    #[inline(always)]
    #[must_use]
    fn from(op: u8) -> Self {
        Self(op)
    }
}
impl From<LinkedByte> for u8 {
    #[inline(always)]
    #[must_use]
    fn from(op: LinkedByte) -> Self {
        op.0
    }
}
impl core::fmt::Debug for LinkedByte {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        let mut ds = fmt.debug_tuple("LinkedByte");
        ds.field(&self.value());
        ds.field(&
            if self.is_linked() {"link"} else {"end"}
        );
        ds.finish()
    }
}