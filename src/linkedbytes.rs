//! The Linked Bytes format, capable of storing arbitrarily large unsigned integers, also used to efficently store Unicode strings.
//!
//! If you only want non-negative integers, you should stick to this format. (Signed LB integers are also planned.) Otherwise, use either Head Byte or Extended Head Byte.

use core::slice::SliceIndex;
use alloc::vec::Vec;

/// The `Result` specialization for the methods converting iterators/arrays of bytes into instances of `LBNum`.
pub type DecodeResult = Result<LBNum, InvalidLBSequence>;

/// A number in the Linked Bytes format, capable of storing arbitrarily large non-negative integers.
///
/// See the [module-level documentation][modlb] for more.
///
/// [modlb]: index.html "bigbit::linkedbytes — the Linked Bytes format, capable of storing arbitrarily large non-negative integers, also used to efficently store Unicode strings"
#[derive(Clone, Debug)]
pub struct LBNum(pub(crate) LBSequence);

impl LBNum {
    /// The zero value.
    ///
    /// This does not allocate memory.
    pub const ZERO: Self = Self(LBSequence::empty());
    /// The amount of bytes used in the number.
    #[inline(always)]
    #[must_use]
    pub fn num_bytes(&self) -> usize {
        self.0.inner().len()
    }
    /// Increments the value.
    #[inline(always)]
    pub fn increment(&mut self) {
        self.increment_at_index(0);
    }
    #[inline(always)]
    pub fn decrement(&mut self) {
        assert!(self.checked_decrement())
    }
    /// Decrements the value, returning `true` if the decrement did anything and `false` if `self` was zero.
    #[inline(always)]
    pub fn checked_decrement(&mut self) -> bool {
        use crate::ops::linkedbytes::DecrementResult;
        match self.decrement_at_index(0) {
            DecrementResult::EndedWithBorrow | DecrementResult::NoSuchIndex => false,
            DecrementResult::Ok(_) => true
        }
    }
    #[inline(always)]
    #[must_use]
    pub const fn inner(&self) -> &LBSequence {
        &self.0
    }
    /// Consumes the number and returns its inner sequence.
    #[inline(always)]
    #[must_use]
    pub fn into_inner(self) -> LBSequence {
        self.0
    }
    /// Returns an iterator over the linked bytes, in **little**-endian byte order.
    #[inline(always)]
    pub fn iter_le(&self) -> impl Iterator<Item = LinkedByte> + DoubleEndedIterator + '_ {
        self.0.iter_le()
    }
    /// Returns an iterator over the linked bytes, in **big**-endian byte order.
    #[inline(always)]
    pub fn iter_be(&self) -> impl Iterator<Item = LinkedByte> + DoubleEndedIterator + '_ {
        self.0.iter_be()
    }

    /// Creates an `LBNum` from an `LBSequence`, correcting any and all incorrect bytes into their valid state. **Little-endian byte order is assumed, regardless of platform.**
    ///
    /// If any of the bytes except for the last one are endpoints (most significant bit cleared), they are converted into linked (most significant bit set), and if the last byte is linked, it's converted into and endpoint.
    #[inline(always)]
    pub fn from_sequence(mut op: LBSequence) -> Self {
        Self::fix_in_place(&mut op.inner_mut());
        Self(op)
    }

    /// Ensures that the last element is an endpoint.
    pub(crate) fn ensure_last_is_end(&mut self) {
        if let Some(last) = self.0.inner_mut().last_mut() {
            *last = last.into_end()
        } else {}
    }
    /// Converts the last element to a linked byte, for adding together two `LBNum`s.
    pub(crate) fn convert_last_to_linked(&mut self) {
        if let Some(last) = self.0.inner_mut().last_mut() {
            *last = last.into_linked()
        } else {}
    }

    /// Checks whether the operand is a compliant LB sequence.
    ///
    /// See [`InvalidLBSequence`][0] for reasons why it might not be compliant.
    ///
    /// [0]: struct.InvalidLBSequence.html "InvalidLBSequence — marker error type representing that the decoder has encountered an invalid Linked Bytes sequence"
    pub fn check_slice(op: &[LinkedByte]) -> bool {
        if let Some(last) = op.last() {
            // Sike, the last element is not an endpoint so we can skip the entire thing.
            if last.is_linked() {return false;}
        } else {
            // Zero sequences are empty.
            return true;
        }
        // After the cache residency of `op` was introduced in the previous check, just fuzz through the rest of the elements.
        for el in &op[0..(op.len() - 1)] {
            if el.is_end() {return false;} // Invalid byte detected, lethal force engaged.
        }
        true // ok we're fine
    }
    /// Makes a slice of `LinkedByte`s suitable for storage in a `HBNum` by marking the last byte as an endpoint and the rest as linked ones.
    pub fn fix_in_place(op: &mut [LinkedByte]) {
        if let Some(last) = op.last_mut() {
            // Fix the last element in place.
            if last.is_linked() {*last = last.into_end();}
        } else {
            // We're already finished, it's a valid zero.
            return;
        }
        let end = op.len() - 1;
        for el in &mut op[0..end] {
            if el.is_end() {*el = el.into_linked();}
        }
    }

    /// Removes trailing zeros.
    pub(crate) fn zero_fold(&mut self) {
        for i in (0..self.num_bytes()).rev() {
            if self.0.inner()[i].into_end() == LinkedByte::ZERO_END {
                self.0.inner_mut().pop();
            }
        }
    }
}
impl core::convert::TryFrom<Vec<LinkedByte>> for LBNum {
    type Error = InvalidLBSequence;

    /// Converts a `Vec<LinkedByte>` into an `LBNum`. **Little-endian byte order is assumed, regardless of platform.**
    ///
    /// # Errors
    /// See [`InvalidLBSequence`][0].
    ///
    /// [0]: struct.InvalidLBSequence.html "InvalidLBSequence — marker error type representing that the decoder has encountered an invalid Linked Bytes sequence"
    #[inline]
    fn try_from(op: Vec<LinkedByte>) -> DecodeResult {
        if Self::check_slice(&op) {
            Ok(Self(LBSequence::from(op)))
        } else {
            Err(InvalidLBSequence)
        }
    }
}
impl core::convert::TryFrom<LBSequence> for LBNum {
    type Error = InvalidLBSequence;

    /// Converts an `LBSequence` into an `LBNum`. **Little-endian byte order is assumed, regardless of platform.**
    ///
    /// # Errors
    /// See [`InvalidLBSequence`][0].
    ///
    /// [0]: struct.InvalidLBSequence.html "InvalidLBSequence — marker error type representing that the decoder has encountered an invalid Linked Bytes sequence"
    #[inline]
    fn try_from(op: LBSequence) -> DecodeResult {
        if Self::check_slice(&op.inner()) {
            Ok(Self(op))
        } else {
            Err(InvalidLBSequence)
        }
    }
}
impl core::iter::FromIterator<LinkedByte> for LBNum {
    /// Converts an iterator over linked bytes into an LBNum. **Little-endian byte order is assumed, regardless of platform.**
    ///
    /// If any of the bytes except for the last one are endpoints (most significant bit cleared), they are converted into linked (most significant bit set), and if the last byte is linked, it's converted into and endpoint.
    ///
    /// If possible, use `TryFrom` or `from_sequence` instead.
    fn from_iter<T: IntoIterator<Item = LinkedByte>>(op: T) -> Self {
        let mut resulting_vec = op.into_iter().collect::<Vec<LinkedByte>>();
        Self::fix_in_place(&mut resulting_vec);
        Self(LBSequence::from(resulting_vec))
    }
}
impl core::convert::AsRef<[LinkedByte]> for LBNum {
    #[inline(always)]
    fn as_ref(&self) -> &[LinkedByte] {
        self.0.inner()
    }
}
impl alloc::borrow::Borrow<[LinkedByte]> for LBNum {
    #[inline(always)]
    fn borrow(&self) -> &[LinkedByte] {
        self.0.inner()
    }
}

/// A Linked Bytes number behind a reference.
///
/// The borrow checker ensures that the inner data will **never** be an invalid LB sequence, meaning that after the `TryFrom` check has passed, there's no way that any external code will tamper the borrowed slice.
#[derive(Copy, Clone, Debug)]
pub struct LBNumRef<'a> (&'a [LinkedByte]);
impl<'a> LBNumRef<'a> {
    /// Constructs an `LBNumRef` referring to the specified Linked Byte slice.
    #[inline(always)]
    pub const fn new(op: &'a [LinkedByte]) -> Self {
        Self(op)
    }

    /// Converts an `LBNumRef` into an owned `LBNumRef`. **This dereferences and clones the contents.**
    #[inline(always)]
    pub fn into_owned(self) -> LBNum {
        LBNum::from_sequence(LBSequence::from(self.0))
    }
    /// Returns the inner Linked Byte buffer.
    #[inline(always)]
    pub const fn inner(self) -> &'a [LinkedByte] {
        self.0
    }
}
impl<'a> From<&'a LBNum> for LBNumRef<'a> {
    #[inline(always)]
    fn from(op: &'a LBNum) -> Self {
        Self(op.inner().inner())
    }
}
impl<'a> core::convert::TryFrom<&'a [LinkedByte]> for LBNumRef<'a> {
    type Error = InvalidLBSequence;

    #[inline(always)]
    fn try_from(op: &'a [LinkedByte]) -> Result<Self, InvalidLBSequence> {
        if LBNum::check_slice(op) {
            Ok(Self(op))
        } else {
            Err(InvalidLBSequence)
        }
    }
}
impl<'a> core::ops::Deref for LBNumRef<'a> {
    type Target = [LinkedByte];
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

// Implementations for arithmetic operations are located in crate::ops::linkedbytes.

/// Marker error type representing that the decoder has encountered an invalid Linked Bytes sequence, created by the `TryFrom` implementation of `LBNum`.
///
/// The only reason for this to ever happen is incorrect state of the link bit in one of the bytes: all the bytes except for the last one **have to be linked** (most significant bit set), and the last one **has to be an endpoint** (most significant bit clear).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct InvalidLBSequence;

/// An owned unchecked Linked Bytes sequence, used for storing either strings or numbers.
#[derive(Clone, Debug)]
pub struct LBSequence(pub(crate) Vec<LinkedByte>);
impl LBSequence {
    /// Creates an empty `LBSequence`.
    #[inline(always)]
    #[must_use]
    pub const fn empty() -> Self {
        Self(Vec::new())
    }

    /// Immutably borrows the inner container.
    #[inline(always)]
    #[must_use]
    pub const fn inner(&self) -> &Vec<LinkedByte> {
        &self.0
    }
    /// Mutably borrows the inner container.
    #[inline(always)]
    #[must_use]
    pub fn inner_mut(&mut self) -> &mut Vec<LinkedByte> {
        &mut self.0
    }
    /// Performs slice indexing.
    ///
    /// See [the standard library documentation][0] for more.
    ///
    /// [0]: https://doc.rust-lang.org/std/primitive.slice.html#method.get "std::slice::get — returns a reference to an element or subslice depending on the type of index"
    #[inline(always)]
    pub fn get<I: SliceIndex<[LinkedByte]>>(&self, index: I) -> Option<&<I as SliceIndex<[LinkedByte]>>::Output> {
        self.inner().get(index)
    }
    /// Performs mutable slice indexing.
    ///
    /// See [the standard library documentation][0] for more.
    ///
    /// [0]: https://doc.rust-lang.org/std/primitive.slice.html#method.get_mut "std::slice::get_mut — returns a mutable reference to an element or subslice depending on the type of index"
    #[inline(always)]
    pub fn get_mut<I: SliceIndex<[LinkedByte]>>(&mut self, index: I) -> Option<&mut <I as SliceIndex<[LinkedByte]>>::Output> {
        self.inner_mut().get_mut(index)
    }
    /// Returns the number of bytes in the sequence.
    #[inline(always)]
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// Returns `true` if the sequence is empty (= 0) or `false` otherwise.
    #[inline(always)]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns an iterator over the linked bytes in **little**-endian byte order.
    #[inline(always)]
    pub fn iter_le(&self) -> impl Iterator<Item = LinkedByte> + DoubleEndedIterator + '_ {
        self.0.iter().copied()
    }
    /// Returns an iterator over the linked bytes in **big**-endian byte order.
    #[inline(always)]
    pub fn iter_be(&self) -> impl Iterator<Item = LinkedByte> + DoubleEndedIterator + '_ {
        self.iter_le().rev()
    }
    /// Returns an iterator over **mutable references** to the linked bytes in **little**-endian byte order.
    #[inline(always)]
    pub fn iter_mut_le(&mut self) -> impl Iterator<Item = &mut LinkedByte> + DoubleEndedIterator + '_ {
        self.0.iter_mut()
    }
    /// Returns an iterator over **mutable references** to the linked bytes in **big**-endian byte order.
    #[inline(always)]
    pub fn iter_mut_be(&mut self) -> impl Iterator<Item = &mut LinkedByte> + DoubleEndedIterator + '_ {
        self.iter_mut_le().rev()
    }
}
impl From<Vec<LinkedByte>> for LBSequence {
    #[inline(always)]
    #[must_use]
    fn from(op: Vec<LinkedByte>) -> Self {
        Self(op)
    }
}
impl From<&[LinkedByte]> for LBSequence {
    /// Clones the contens of the slice into a Linked Bytes sequence.
    #[inline(always)]
    #[must_use]
    fn from(op: &[LinkedByte]) -> Self {
        Self(Vec::from(op))
    }
}
impl AsRef<[LinkedByte]> for LBSequence {
    #[inline(always)]
    fn as_ref(&self) -> &[LinkedByte] {
        self.0.as_ref()
    }
}
impl alloc::borrow::Borrow<[LinkedByte]> for LBSequence {
    #[inline(always)]
    fn borrow(&self) -> &[LinkedByte] {
        &self.0
    }
}
impl AsMut<[LinkedByte]> for LBSequence {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut [LinkedByte] {
        self.0.as_mut()
    }
}
impl alloc::borrow::BorrowMut<[LinkedByte]> for LBSequence {
    #[inline(always)]
    fn borrow_mut(&mut self) -> &mut [LinkedByte] {
        &mut self.0
    }
}
impl core::iter::FromIterator<LinkedByte> for LBSequence {
    /// Converts an iterator over linked bytes into an `LBSequence`. **Little-endian byte order is assumed, regardless of platform.**
    #[inline(always)]
    #[must_use]
    fn from_iter<T: IntoIterator<Item = LinkedByte>>(op: T) -> Self {
        Self(op.into_iter().collect::<Vec<LinkedByte>>())
    }
}

/// An element in a series of Linked Bytes.
#[repr(transparent)]
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

    /// The smallest value representable, as a `u8`.
    ///
    /// Alias for `ZERO_END`.
    pub const MIN: u8 = 0;
    /// The largest value representable, as a `u8`.
    ///
    /// The value is `127`, since the individual Linked Bytes are 7-bit, plus the link/end flag.
    pub const MAX: u8 = 127;

    /// Returns `true` if the byte is linked to a following byte, `false` otherwise.
    ///
    /// The opposite of `is_end`.
    #[inline(always)]
    #[must_use]
    pub const fn is_linked(self) -> bool {
        (self.0 & Self::LINK_MASK) != 0
    }
    /// Returns `true` if the byte is **not** linked to a following byte (i.e. is an endpoint), `false` otherwise.
    ///
    /// The opposite of `is_linked`.
    #[inline(always)]
    #[must_use]
    pub const fn is_end(self) -> bool {
        (self.0 & Self::LINK_MASK) == 0
    }
    /// Returns the value of the linked byte, in the range from 0 to 127, inclusively.
    ///
    /// The main use for this is performing arithmetic with linked bytes.
    #[inline(always)]
    #[must_use]
    pub const fn value(self) -> u8 {
        self.0 & Self::VALUE_MASK
    }
    /// Sets the link bit to `true` (linked state).
    #[inline(always)]
    #[must_use]
    pub const fn into_linked(self) -> Self {
        Self(self.0 | Self::ZERO_LINK.0)
    }
    /// Converts `self` into the linked state **in place**.
    #[inline(always)]
    pub fn make_linked(&mut self) {
        *self = self.into_linked()
    }
    /// Sets the link bit to `false` (endpoint state).
    #[inline(always)]
    #[must_use]
    pub const fn into_end(self) -> Self {
        Self(self.0 & Self::VALUE_MASK)
    }
    /// Converts `self` into the linked state **in place**.
    #[inline(always)]
    pub fn make_end(&mut self) {
        *self = self.into_end()
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
    /// Performs checked wrapping addition. Unlike [`checked_add`][ca], this method returns a tuple, in which the first value is the result, which wraps over if the result overflows the limit of 127, and the second value is whether the overflow actually occurred.
    ///
    /// [ca]: #method.checked_add "checked_add — perform checked addition"
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

    /// Consumes the value and unwraps it into its inner `u8`, retaining the link bit if it's set.
    ///
    /// Use [`into_int7`][ii7] if you need only the value without the link bit, which is usually the case.
    ///
    /// [ii7]: method.into_int7 "into_int7 — consume the value and unwrap it into its inner 7-bit integer"
    #[inline(always)]
    #[must_use]
    pub fn into_inner(self) -> u8 {
        self.0
    }
    /// Consumes the value and unwraps it into its inner 7-bit integer, i.e. dropping the link bit if it's set.
    ///
    /// Use [`into_inner`][ii] if you need the unmodified value with the link bit unmodified.
    ///
    /// [ii]: #method.into_inner "into_inner — consume the value and unwrap it into its inner u8, retaining the link bit if it's set"
    #[inline(always)]
    #[must_use]
    pub fn into_int7(self) -> u8 {
        self.into_end().0
    }
}
impl From<u8> for LinkedByte {
    /// Constructs an endpoint byte from a `u8`.
    ///
    /// The most significant bit is silently dropped. Use `into_linked` to convert the result into a linked byte if you actually want to initialize it like that.
    #[inline(always)]
    #[must_use]
    fn from(op: u8) -> Self {
        Self(op).into_end()
    }
}
impl From<(u8, bool)> for LinkedByte {
    /// Constructs either a linked or an endpoint byte depending on the second argument.
    ///
    /// The most significant bit is silently dropped.
    #[inline(always)]
    #[must_use]
    fn from(op: (u8, bool)) -> Self {
        // This casts the boolean into a 0/1 u8 and shifts it into the link bit position, then constructs a mask which either accepts the link bit or sets it.
        let mask = Self::VALUE_MASK | ((op.1 as u8) << 7);
        let result = Self(op.0).into_linked(); // Quickly make it linked to use the mask
        Self(result.0 & mask)
    }
}
impl From<LinkedByte> for u8 {
    /// Consumes the byte and unwraps it into its inner `u8`, retaining the link bit if it's set.
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

use core::cmp::{self, Ordering};
impl cmp::PartialOrd for LinkedByte {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl cmp::Ord for LinkedByte {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        self.into_end().0.cmp(&other.into_end().0)
    }
}