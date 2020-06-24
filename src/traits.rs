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
/// This serves the same purpose as the [`DivRem`] trait, but the division is performed in place (the variable/field containing the dividend is replaced by the quotient), and the only value returned is the remainder.
///
/// [`DivRem`]: trait.DivRem.html "DivRem — combined division and remainder operations"
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
/// ## This is not a general-purpose trait!
/// **Implementing this trait for types outside of the `bigbit` crate is greatly discouraged for semantic purposes.** Use non-trait methods or third-party traits instead.
pub trait Gcd<Rhs = Self>: Sized {
    /// Performs the calculation of the greatest common divisor.
    ///
    /// Most implementations use the [Euclidean algorithm][0] for this.
    ///
    /// [0]: https://en.wikipedia.org/wiki/Euclidean_algorithm "Euclidean Algorithm on Wikipedia"
    fn gcd(lhs: Self, rhs: Rhs) -> Self;
}

/// Performs the `+` operation at the specified coefficient byte of a BigBit number. This is used by the multiplication implementations as a simple and fast way of multiplying something by a power of 128 or 256 (former in the case of Linked Bytes, latter for Head Byte and Extended Byte) and then adding the result to another number.
///
/// The similar concept from the scalar binary integer world is **bit shifting**, performed by the `<<`/`<<=` and `>>`/`>>=` operators. The difference is that the shift here happens on the fly by tweaking indicies, rather than by shifting the entire number and then adding it to another number.
///
/// This operation is not and should not be defined on floating-point numbers, since those have a coefficient which does not meaningfully interact with this concept.
///
/// For an in-place version, see [`AddAssignAt`].
///
/// ## This is not a general-purpose trait!
/// **Implementing this trait for types outside of the `bigbit` crate is greatly discouraged for semantic purposes.** Use non-trait methods or third-party traits instead.
///
/// [`AddAssignAt`]: trait.AddAssignAt.html "AddAssignAt — performs the += operation at the specified coefficient byte of a BigBit number"
pub trait AddAt<Rhs = Self> {
    /// The return type for the operation.
    type Output;
    /// Performs the shifting addition.
    #[must_use = "this is an expensive non-in-place operation"]
    fn add_at(self, index: usize, rhs: Rhs) -> Self::Output;
}
/// Performs the `+=` operation at the specified coefficient byte of a BigBit number. This is used by the multiplication implementations as a simple and fast way of multiplying something by a power of 128 or 256 (former in the case of Linked Bytes, latter for Head Byte and Extended Head Byte) and than adding the result to another number.
///
/// The similar concept from the scalar binary integer world is **bit shifting**, performed by the `<<`/`<<=` and `>>`/`>>=` operators. The difference is that the shift here happens on the fly by tweaking indicies, rather than by shifting the entire number and then adding it to another number.
///
/// This operation is not and should not be defined on floating-point numbers, since those have a coefficient which does not meaningfully interact with this concept.
///
/// For a version which returns the result instead of performing the operation in-place, see [`AddAt`].
///
/// # This is not a general-purpose trait!
/// **Implementing this trait for types outside of the `bigbit` crate is greatly discouraged for semantic purposes.** Use non-trait methods or third-party traits instead.
///
/// [`AddAt`]: trait.AddAt.html "AddAt — performs the + operation at the specified coefficient byte of a BigBit number"
pub trait AddAssignAt<Rhs = Self> {
    /// Performs the shifting addition in-place.
    fn add_assign_at(&mut self, index: usize, rhs: Rhs);
}