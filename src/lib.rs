//! This is an implementation of the [BigBit standard][BigBitStd], used for representing arbitrarily large numbers and strings in a compact way. The only implementation provided by the author is [a Node.js implementation](https://github.com/bigbit/bigbitjs "BigBit.js on GitHub") — this crate aims to implement the functionality presented there with idiomatic Rust code.
//!
//! In short, the specification describes 3 formats:
//! - **Head Byte** (***HB***), used for storing extremely large (but still finitely large) signed integers *and* decimal fractions without losing precision
//! - **Extended Head Byte** (***EHB***), used for storing arbitrarily large signed integers *and* decimal fractions without losing precision as long as sufficent storage space is provided.
//! - **Linked Bytes** (***LB***), used for storing arbitrarily large **unsigned integers**, mainly used directly inside the Extended Head Byte format to store the exponent and the number of additional coefficient bytes, but also useful for storing strings better than both UTF-8 and UTF-16.
//!
//! Since this is a format parser, `#![no_std]` is enabled by default, meaning that `alloc` is the only dependency, allowing you to use this in a freestanding environment.
//!
//! # State
//! Currently, not the entire BigBit standard is implemented. Here's a list of what's already done:
//! - Head Byte number storage
//! - Linked Bytes number storage
//! - Linked Bytes addition and subtraction (`Add`/`AddAssign` and `Sub`/`SubAssign`)
//!
//! And here's a list of what's not finished just yet:
//! - Borrowed Linked Bytes (required for EHB) — a Linked Bytes number which doesn't own its contents and is a slice into an EHB number (will be added in 0.0.x)
//! - Creating HB/LB numbers from primitive integers and `f32`/`f64` (most likely will be added in 0.1.0)
//! - The Extended Header Byte format (will be added in 0.0.x)
//! - Arithmetic operations (addition, subtraction, multiplication and division are all defined by the BigBit standard), except for LB addition, which is already implemented; the main issue is dealing with the exponents (will mark the 1.0.0 release, might be partially added over the course of 0.x.x releases)
//! - Strings encoded using Linked Bytes (will be added in 0.0.x)
//! - `Debug` and `Display` formatting (i.e. converting the numbers either into a debugging-friendly representation as an array of bytes or a string representing the number in decimal scientific notation or full notation, as well as other numeric notations; simple `Debug` and `Display` decimal formatting will be added in 0.1.0 while the rest is planned for 1.0.0)
//! - **Tests** (planned for 0.1.0 but might be partially added earlier)
//!
//! ##### If you're wondering why all versions until `0.0.3` are yanked, these had a minor LB addition bug, so please upgrade if you haven't already.
//!
//! [BigBitStd]: https://github.com/amitguptagwl/BigBit "BitBit specification on GitHub"

#![no_std]
extern crate alloc;

pub mod headbyte;
pub use headbyte::{HBNum, HeadByte};
// Uncomment these after adding EHB:
//pub mod extheadbyte;
//pub use extheadbyte::{EHBNum, ExtHeadByte};
pub mod linkedbytes;
pub use linkedbytes::{LBNum, LinkedByte};

// This module is dedicated to the implementations of `Ord`, `Eq` and arithmetic operations on the BigBit formats. Please implement these there and only there.
mod ops;

/// The sign of a number.
///
/// Either positive or negative. Zero values in BigBit formats are **always positive**, and `NaN` values are **always negative**.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Sign {
    Positive,
    Negative
}
impl From<bool> for Sign {
    /// Treats `true` as negative and `false` as positive.
    #[inline]
    #[must_use]
    fn from(op: bool) -> Self {
        if op {Self::Negative} else {Self::Positive}
    }
}
impl From<Sign> for bool {
    /// Treats `true` as negative and `false` as positive.
    #[inline(always)]
    #[must_use]
    fn from(op: Sign) -> Self {
        match op {
            Sign::Positive => false,
            Sign::Negative => true
        }
    }
}
impl core::fmt::Display for Sign {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        fmt.write_str(match self {
            Sign::Positive => "Positive",
            Sign::Negative => "Negative"
        })
    }
}

/// An exponent for the Head Byte and Extended Head Byte formats.
///
/// To retreive the real value of an [E]HB number, its stored value is multiplied by 10 raised to the power of this value as retreived using [`into_inner`][0].
///
/// [0]: #method.into_inner "into_inner — consumes the value and returns the inner byte"
///
/// This is **not** a 2's complement signed number: it ranges from -127 to +127, having one bit as the sign and the rest as a normal 7-bit unsigned integer. As a consequence, it's possible to store `0b1_0000000` as the exponent, meaning a resulting exponent of 10⁻⁰, which is undefined. In most cases, this transformation is unwanted (that is, accidential, most likely happening because of a serious mistake during bitwise operations), and as such is not allowed, producing a `TryFrom` error.
///
/// In other words, **protection against `-0` is a safety guarantee**, and actually creating an exponent with this value **requires unsafe code**.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Exponent(u8);

impl Exponent {
    /// Wraps a byte into an exponent, ignoring the invalid `0b1_0000000` case.
    ///
    /// This is the unsafe unchecked version of the `TryFrom` implementation.
    ///
    /// # Safety
    /// The value must never be `0b1_0000000` (`-0`), since avoiding that case is a safety guaranteee of the `Exponent` type.
    #[inline(always)]
    #[must_use]
    pub unsafe fn from_u8_unchecked(op: u8) -> Self {
        Self(op)
    }

    /// Consumes the value and returns the inner byte.
    ///
    /// See the struct-level documentation for the meaning of this value.
    #[inline(always)]
    #[must_use]
    pub fn into_inner(self) -> u8 {
        self.0
    }
}
impl core::convert::TryFrom<u8> for Exponent {
    type Error = InvalidExponentError;
    /// Wraps a byte into an exponent.
    ///
    /// # Errors
    /// If the supplied value is `0b1_0000000` (`-0`), `Err(InvalidExponentError)` is returned, where [`InvalidExponentError`][0] is a marker error type.
    ///
    /// [0]: struct.InvalidExponentError.html "InvalidExponentError — the error marker for when 0b10000000 is encountered in the TryFrom implementation of Exponent"
    #[inline(always)]
    fn try_from(op: u8) -> Result<Self, InvalidExponentError> {
        if op == 0b1_0000000 {return Err(InvalidExponentError);}
        Ok(Self(op))
    }
}
impl From<Exponent> for u8 {
    /// Consumes the exponent and returns the underlying inner byte.
    #[inline(always)]
    #[must_use]
    fn from(op: Exponent) -> Self {
        op.0
    }
}
/// The error marker for when `0b10000000` is encountered in the `TryFrom` implementation of [`Exponent`][1].
///
/// [1]: struct.Exponent.html "Exponent — an exponent for the Head Byte format"
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct InvalidExponentError;

/// This is a hack to get around the fact that `debug_struct` only accepts `Debug` formatting rather than `Display`, which becomes a verbosity issue if the values of an enum are known from the name of the field.
#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) struct SignDisplayAsDebug(pub(crate) Sign);
impl core::fmt::Debug for SignDisplayAsDebug {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        core::fmt::Display::fmt(&self.0, fmt)
    }
}

#[cfg(test)]
mod tests;