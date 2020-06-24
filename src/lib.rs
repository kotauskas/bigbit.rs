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
//! Currently, not the entire BigBit standard is implemented, and **the crate is not ready for use in production just yet**. There are also **no stability guarantees whatsoever**. Here's a list of what's already done:
//! - Head Byte number storage (not really finished, just a stub)
//! - Linked Bytes number storage and arithmetic
//! - Converting Linked Bytes to and from primitive integers
//! - Displaying Linked Bytes numbers in binary, octal, decimal and hexadecimal using formatters as well as other bases (arbitrary from 2 to 36) using a dedicated method
//! - Borrowed Linked Bytes (required for EHB) — a Linked Bytes number which doesn't own its contents and is a slice into an EHB number (still a stub)
//!
//! And here's a list of what's not finished just yet:
//! - Creating \[E\]HB numbers from primitive integers and `f32`/`f64` (most likely will be added in 0.1.0)
//! - The Extended Header Byte format (will be added in 0.0.x)
//! - Arithmetic operations (addition, subtraction, multiplication and division are all defined by the BigBit standard) for \[E\]HB; the main issue is dealing with the exponents (will mark the 1.0.0 release, might be partially added over the course of 0.x.x releases)
//! - Strings encoded using Linked Bytes (will be added in 0.0.x)
//! - `Debug` and `Display` formatting for \[E\]HB (i.e. converting the numbers either into a debugging-friendly representation as an array of bytes or a string representing the number in decimal scientific notation or full notation, as well as other numeric notations; simple `Debug` and `Display` decimal formatting will be added in 0.1.0 while the rest is planned for 1.0.0)
//! - **Tests** (planned for 0.1.0 but might be partially added earlier)
//!
//! # Feature flags
//! Several [Cargo feature flags][CargoFeatures] are available:
//! - **`std`** *(enabled by default)* — disables `no_std`, allowing for `std`-dependent trait implementations. **Disable this feature if using `no_std`.**
//! - **`num_traits`** *(enabled by default)* — enables trait implementations for traits from [`num-traits`], disable to insignificantly decrease compile time and code size. **The current version of `num-traits` is `0.2.x` — please open an issue if a new one comes out.**
//! - **`clippy`** *(enabled by default)* — disable to remove all mentions of Clippy lints to avoid unknown lint errors if working on this crate without Clippy installed.
//!
//! # Changelog
//! The full version history can be found [here][changelog].
//!
//! [BigBitStd]: https://github.com/amitguptagwl/BigBit "BitBit specification on GitHub"
//! [changelog]: https://github.com/kotauskas/bigbit.rs/releases " "
//! [CargoFeatures]: https://doc.rust-lang.org/cargo/reference/features.html "Documentation for crate features on the Cargo Reference"
//! [`num-traits`]: https://crates.io/crates/num-traits "num-traits on Crates.io"

#![cfg_attr(feature = "clippy", warn(clippy::pedantic, clippy::nursery))]
#![cfg_attr(feature = "clippy", allow( // All of these lints are generally bullshit and should not be a thing or require serious improvement.
    clippy::suspicious_arithmetic_impl,
    clippy::suspicious_op_assign_impl,
    clippy::inline_always,
    clippy::large_digit_groups,
    clippy::doc_markdown,
    clippy::must_use_candidate,
    clippy::match_bool,
    clippy::wildcard_imports,
    clippy::redundant_pub_crate,
    clippy::if_not_else,
    clippy::cast_lossless, // What does this even mean?
))]

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

pub mod headbyte;
pub use crate::headbyte::{HBNum, HeadByte};
// Uncomment these after adding EHB:
//pub mod extheadbyte;
//pub use extheadbyte::{EHBNum, ExtHeadByte};
pub mod linkedbytes;
pub use linkedbytes::{LBNum, LBString, LinkedByte};

mod traits;
pub use traits::*;

#[cfg(feature = "num_traits")]
pub extern crate num_traits;

pub(crate) mod tables; pub(crate) use tables::*;

/// Basic types which can and should be in scope when using BigBit.
pub mod prelude {
    pub use crate::linkedbytes::{LBNum, LBString, LinkedByte};
    pub use crate::headbyte::{HBNum, HeadByte};
}

/// Calculates the greatest common divisor of two numbers.
///
/// This is an alias for using the trait directly which allows you to write `bigbit::gcd(<any two BigBit numbers>)` instead of importing the trait into scope and using `A::gcd(b)`, which is less readable and less functional-styled.
#[inline(always)]
pub fn gcd<A, B>(lhs: A, rhs: B) -> A
where A: Gcd<B> {
    A::gcd(lhs, rhs)
}

/// The sign of a number.
///
/// Either positive or negative. Zero values in BigBit formats are **always positive**, and `NaN` values are **always negative**.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Sign {
    Positive = 0,
    Negative = 1,
}
impl From<bool> for Sign {
    /// Treats `true` as negative and `false` as positive.
    #[inline]
    fn from(op: bool) -> Self {
        if op {Self::Negative} else {Self::Positive}
    }
}
impl From<Sign> for bool {
    /// Treats `true` as negative and `false` as positive.
    #[inline(always)]
    fn from(op: Sign) -> Self {
        match op {
            Sign::Positive => false,
            Sign::Negative => true,
        }
    }
}
impl core::fmt::Display for Sign {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        fmt.write_str(match self {
            Self::Positive => "Positive",
            Self::Negative => "Negative",
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