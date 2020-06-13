//! A Unicode string format implemented using Linked Bytes.
//!
//! This module is the home for [`LBString`][lbs] and [`LBCharsIter`][lbci], which implement the Linked Bytes string storage format, as seen in the official specification. This is, perhaps, the most widely used feature of BigBit, since it's useful even when you don't need the powerful big number storage. The documentation on the `LBString` page elaborates on that.
//!
//! [lbs]: struct.LBString.html "LBString — a Unicode string stored using the Linked Bytes format"
//! [lbci]: struct.LBCharsIter.html "LBCharsIter — an iterator over the codepoints in an LBString"

use super::{LBNum, LBSequence, LBNumRef};

/// A Unicode string stored using the Linked Bytes format.
///
/// This is more compact than all of the current UTF formats (namely, UTF-1, 7, 8, 16, let alone 32), since no surrogate pairs are used. Instead, the Linked Bytes format is leveraged, with separate codepoints being stored as individual Linked Bytes numbers. Both the link/end bits of the bytes and length of the entire message, either via the null terminator (which still works since a linking 0 has the most significant bit set to 1 and cannot be confused with the null terminator when reinterpreted as `u8`) or via storing it separately (as Rust `String`s do), are available. This means that the UTF-32 number of each codepoint can be encoded using the usual Linked Bytes format, with the link bit cleared in a byte indicating that one character has ended and a new one is coming next.
///
/// # Usage
/// Conversion from `String` or `&str`:
/// ```
/// # extern crate alloc;
/// # use alloc::string::String;
/// # use bigbit::LBString;
/// static MY_STRING: &str = "My string!";
/// let stdstring = String::from("This is a standard string!");
///
/// let my_string_lb = LBString::from(MY_STRING); // Creates an LBString from a string slice
/// let stdstring_lb = LBString::from(stdstring); // Creates an LBString from a String
/// let my_string_lb_2 = MY_STRING.chars().collect::<LBString>(); // Creates an LBString from an iterator
///
/// # assert_eq!(String::from(my_string_lb), MY_STRING);
/// # assert_eq!(String::from(stdstring_lb), "This is a standard string!");
/// # assert_eq!(String::from(my_string_lb_2), MY_STRING);
/// ```
#[derive(Clone, Debug)]
pub struct LBString(LBSequence);
impl LBString {
    /// Returns an iterator over the codepoints in the string.
    ///
    /// This is the core method of this type. Most other methods use this to perform more complex operations, such as conversion from an `&str`.
    #[inline(always)]
    pub fn chars(&self) -> impl Iterator<Item = char> + '_ {
        LBCharsIter::new(self)
    }

    /// Counts the number of **codepoints** stored.
    ///
    /// This will iterate through the entire string and count how many codepoints were resolved successfully. Currently, this is implemented as simply `self.chars().count()`.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.chars().count()
    }
    /// Returns `true` if there are no codepoints stored, `false` otherwise.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty() // We can use the container length, since if it's 0, then it's pointless to try to iterate, otherwise there's guaranteed to be a codepoint.
    }
    /// Returns an immutable reference to the underlying sequence.
    #[inline(always)]
    pub const fn inner(&self) -> &LBSequence {
        &self.0
    }
}
impl core::iter::FromIterator<char> for LBString {
    fn from_iter<I: IntoIterator<Item = char>>(iter: I) -> Self {
        let mut result = Self(LBSequence::empty());
        let mut lbn = LBNum::ZERO;
        for c in iter {
            lbn.make_zero(); // This is a specialized method for making the value zero without reallocating,
                             // which makes it vital for larger strings.
            lbn += u32::from(c);
            result.0.inner_mut().extend(lbn.iter_le());
        }
        result
    }
}
impl<'a> core::iter::FromIterator<&'a char> for LBString {
    /// Convenience implementation for collections which iterate over references to items rather than the items themselves, to avoid repetitive `.copied()` in calling code.
    #[inline(always)]
    fn from_iter<I: IntoIterator<Item = &'a char>>(iter: I) -> Self {
        iter.into_iter().copied().collect::<Self>()
    }
}
impl core::fmt::Display for LBString {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        use core::fmt::Write;
        for c in self.chars() {
            if let Err(e) = f.write_char(c) {return Err(e);} // Stop right where we are if we can't write anything.
        }
        Ok(())
    }
}
/// An iterator over the codepoints in an `LBString`.
///
/// This resolves the codepoints on the fly, as all lazy iterators do. Thus creating such an iterator is totally free.
///
/// The values are **not checked when resolving,** meaning that any invalid Unicode codepoints will be carried over into the result. The reason is that the validity of the values is ensured by the `LBString` type during creation. This means that any unsafe code which incorrectly modifies an `LBString` will most likely trigger a panic or an infinite loop.
pub struct LBCharsIter<'a> {
    inner: &'a LBString,
    index: usize
}
impl<'a> LBCharsIter<'a> {
    pub const fn new(s: &'a LBString) -> Self {
        Self {inner: s, index: 0}
    }
}
impl<'a> Iterator for LBCharsIter<'a> {
    type Item = char;
    fn next(&mut self) -> Option<char> { // If anything breaks, blame this tymethod (seriously, please do).
        use core::{convert::TryInto, hint::unreachable_unchecked};
        let mut chosen_range = self.index..self.index;
        loop {
            if let Some(v) = self.inner.inner().get(self.index) {
                self.index += 1;
                chosen_range.end = self.index;
                if v.is_end() {break;}
            } else {
                return None;
            }
        }
        // inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner
        // inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner
        // inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner
        // inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner inner
        let refnum = TryInto::<LBNumRef>::try_into(&self.inner.inner().inner()[chosen_range])
            .unwrap_or_else(|_| unsafe {unreachable_unchecked()}); // Value validity is a safety guarantee for LBString, which is why we can simply
                                                                   // invoke UB if it fails. Great!
        let codepoint = TryInto::<u32>::try_into(refnum)
            .unwrap_or_else(|_| unsafe {unreachable_unchecked()}); // Same thing here.
        let codepoint = TryInto::<char>::try_into(codepoint)
            .unwrap_or_else(|_| unsafe {unreachable_unchecked()}); // And here.
        Some(codepoint)
    }
}