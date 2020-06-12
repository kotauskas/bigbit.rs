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
//! # Changelog
//! The full version history can be found [here][changelog].
//!
//! [BigBitStd]: https://github.com/amitguptagwl/BigBit "BitBit specification on GitHub"
//! [changelog]: https://github.com/kotauskas/bigbit.rs/releases " "

#![cfg_attr(feature="clippy", allow(clippy::suspicious_op_assign_impl))]

#![no_std]
extern crate alloc;

pub mod headbyte;
pub use crate::headbyte::{HBNum, HeadByte};
// Uncomment these after adding EHB:
//pub mod extheadbyte;
//pub use extheadbyte::{EHBNum, ExtHeadByte};
pub mod linkedbytes;
pub use linkedbytes::{LBNum, LBString, LinkedByte};

pub(crate) mod tables; pub(crate) use tables::*;

/// Basic types which can and should be in scope when using BigBit.
pub mod prelude {
    pub use crate::linkedbytes::{LBNum, LBString, LinkedByte};
    pub use crate::headbyte::{HBNum, HeadByte};
}

/// Combined division and remainder operations.
///
/// The usual process for getting both the division result and the remainder is performing both operations sequentially, since most programming languages, including Rust, do not provide an interface for introspecting the remainder acquired after dividing with the designated CPU instructions (LLVM will, however, optimize the two operations into one, given that optimizations are actually enabled). With BitBit numbers, however, division is implemented iteratively, without harware acceleration, i.e. the CPU doesn't assist the process by providing special instructions to perform the operations.
///
/// The current division implementation simply drops the remainder, which has the cost of deallocating the coefficient/`LinkedByte` sequence, only to allocate and calculate it yet again. By using this trait, you ensure that nothing is ever dropped during division and that you get both the quotient and remainder the fastest way possible.
pub trait DivRem<Rhs = Self>: Sized {
    /// The type for the quotient.
    type Quotient;
    /// The type for the remainder.
    type Remainder;
    /// Performs combined quotient and remainder calculation.
    ///
    /// The first element is the quotient, and the second one is the remainder.
    fn div_rem(self, rhs: Rhs) -> (Self::Quotient, Self::Remainder);
}
/// Division in place combined with retreiving the remainder.
///
/// This serves the same purpose as the [`DivRem`][dr] trait, but the division is performed in place (the variable/field containing the dividend is replaced by the quotient), and the only value returned is the remainder.
///
/// [dr]: trait.DivRem.html "DivRem — combined division and remainder operations"
pub trait DivRemAssign<Rhs = Self> {
    /// The type for the remainder.
    type Remainder;
    /// Performs combined quotient and remainder calculation, returning the remainder and setting the left operand to the quotient.
    fn div_rem_assign(&mut self, rhs: Rhs) -> Self::Remainder;
}

/// Calculating the greatest common divisor.
///
/// The exact signature of this trait is designed specifically for BigBit types (or any other integer types which own a memory allocation, for that matter), in that it takes both operands by value. For `Copy` types this is nothing other than an advantage; for the memory allocated integer types we're dealing here, it's a matter of cloning the numbers before the operation.
///
/// Until specialization becomes stable, not implementing this trait transitively is a logic error rather than a scenario which is protected against by a default blanket implementation. **In short, if you implement `Gcd<U>` for type `T`, you need to also implement `Gcd<T>` for `U`. It's a viable option to do that by writing an `#[inline(always)]` shim which calls `T::gcd(value_of_u)`.
///
/// **Implementing this trait for types outside of the `bigbit` crate is greatly discouraged.** Use non-trait methods or third-party traits for those purposes.
pub trait Gcd<Rhs = Self>: Sized {
    /// Performs the calculation of the greatest common divisor.
    ///
    /// Most implementations use the [Euclidean algorithm][0] for this.
    ///
    /// [0]: https://en.wikipedia.org/wiki/Euclidean_algorithm "Euclidean Algorithm on Wikipedia"
    fn gcd(lhs: Self, rhs: Rhs) -> Self;
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
            Sign::Negative => true,
        }
    }
}
impl core::fmt::Display for Sign {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        fmt.write_str(match self {
            Sign::Positive => "Positive",
            Sign::Negative => "Negative",
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