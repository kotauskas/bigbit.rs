//! # BigBit — compact storage of arbitrarily large numbers
//!
//! This is an implementation of the [BigBit standard][BigBitStd], used for representing arbitrarily large numbers and strings in a compact way. The only implementation provided by the author is [a Node.js implementation](https://github.com/bigbit/bigbitjs "BigBit.js on GitHub") — this crate aims to implement the functionality presented there with idiomatic Rust code.
//!
//! Since this is a format parser, `#![no_std]` is enabled by default, meaning that `alloc` is the only dependency, allowing you to use this in a freestanding environment.
//!
//! # State
//! Currently, not the entire BigBit standard is implemented. Here's a list of what's already done:
//! - Head Byte number storage
//! - Linked Bytes number storage
//! - Linked Bytes addition (both `Add` and `AddAssign`)
//!
//! And here's a list of what's not finished just yet:
//! - Creating HB/LB numbers from primitive integers and `f32`/`f64` (most likely will be added in 0.1.0)
//! - The Extended Header Byte format (will be added in 0.0.1)
//! - Arithmetic operations (addition, subtraction, multiplication and division are all defined by the BigBit standard), except for LB addition, which is already implemented; the main issue is dealing with the exponents (will mark the 1.0.0 release, might be partially added over the course of 0.x.x releases)
//! - Strings encoded using Linked Bytes (will be added in 0.0.1)
//! - `Debug` and `Display` formatting (i.e. converting the numbers either into a debugging-friendly representation as an array of bytes or a string representing the number in decimal scientific notation or full notation, as well as other numeric notations; simple `Debug` and `Display` decimal formatting will be added in 0.1.0 while the rest is planned for 1.0.0)
//! - **Tests** (planned for 0.1.0 but might be partially added earlier)
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