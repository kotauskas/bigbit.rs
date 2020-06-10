//! The Head Byte format, capable of storing integers and decimal fractions up to ±1.34078079e+281.
//!
//! It's recommended to use this format instead of Extended Head Byte if you're accepting numbers from potentially untrusted locations, since Head Byte imposes a size limit (which is still extremely big, suiting most use cases) while Extended Head Byte does not.

mod ops;

use crate::Sign;
use alloc::vec::Vec;

/// The Head Byte format, capable of storing integers and decimal fractions up to ±1.34078079e+281.
///
/// See the [module-level documentation][modhb] for more.
///
/// [modhb]: index.html "bigbit::headbyte — the Head Byte format, capable of storing integers and fractions up to ±1.34078079e+281"
#[derive(Clone)]
pub struct HBNum {
    hb: HeadByte,
    exponent: Option<Exponent>,
    coefficients: Vec<u8>
}
impl HBNum {
    pub fn from_raw_parts(hb: HeadByte, exponent: Option<Exponent>, coefficients: Vec<u8>) -> Self {
        Self {hb, exponent, coefficients}
    }

    /// Returns the head byte.
    ///
    /// The results of inspecting the head byte are reliable and always match the properties of the actual value of the entire number. For example, if `has_exponent` on the head byte always returns `true`, there always is an exponent byte to retreieve.
    #[inline(always)]
    #[must_use]
    pub fn headbyte(&self) -> HeadByte {
        self.hb
    }
    /// Returns the exponent, or `None` if it's not used (mainly the case for integers).
    #[inline(always)]
    #[must_use]
    pub fn exponent(&self) -> Option<Exponent> {
        self.exponent
    }

    /// Returns an iterator over the coefficients in little endian byte order.
    #[inline(always)]
    pub fn coefficient_le_iter(&self) -> impl Iterator<Item = u8> + DoubleEndedIterator + '_ {
        self.coefficients.iter().copied()
    }
    /// Returns an iterator over the coefficients in big endian byte order.
    #[inline(always)]
    pub fn coefficient_be_iter(&self) -> impl Iterator<Item = u8> + DoubleEndedIterator + '_ {
        self.coefficient_le_iter().rev()
    }
}

/// The Head Byte itself, containing information about the sign, presence of the exponent and the number of coefficients.
///
/// Follows the newtype pattern, meaning that it can be unwrapped into the inner byte.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct HeadByte(u8);

impl HeadByte {
    /// The sign bit mask.
    ///
    /// Use [`sign`][0] to easily retreive the sign.
    ///
    /// [0]: #method.sign "sign — retreives the sign from a Head Byte"
    pub const SIGN_MASK: u8 = 0b10_000000;
    /// The mask used for retreiving the absolute value of the number.
    ///
    /// Use [`abs`][0] or bitwise-`AND` (`&`) this with the Head Byte to retreive the absolute value.
    ///
    /// [0]: #method.abs "abs — retreives the absolute value from the number whose Head Byte is self"
    pub const ABS_MASK: u8 = !Self::SIGN_MASK;
    /// The exponent presence bit mask.
    ///
    /// Use [`has_exponent`][0] to easily retreive this.
    ///
    /// [0]: #method.has_exponent "has_exponent — checks whether the Head Byte is supposed to be followed by an exponent byte"
    pub const HAS_EXPONENT_MASK: u8 = 0b01_000000;
    /// Mask for the number of coefficients.
    ///
    /// Use [`num_coefficients`][0] to easily retreive this.
    ///
    /// [0]: #method.num_coefficients "num_coefficients — retreives the number of following coefficients from a Head Byte"
    pub const NUM_COEFFICIENTS_MASK: u8 = 0b00_111111;
    /// The **∞** (positive infinity) value for the Head Byte.
    ///
    /// No following exponent or coefficients are allowed in this case, despite the exponent bit being set.
    pub const INFINITY: Self = Self(0b01_000000);
    /// The **-∞** (negative infinity) value for the Head Byte.
    ///
    /// No following exponent or coefficients are allowed in this case, despite the exponent bit being set.
    pub const NEG_INFINITY: Self = Self(0b11_000000);
    /// The zero value. There's no distinction between positive and negative zero.
    ///
    /// No following exponent or coefficients are allowed.
    pub const ZERO: Self = Self(0);
    /// The NaN (Not-a-Number) value. There's no distinction between negative/positive NaN or signalling/quiet NaN. (This implementation always generates quiet NaN.)
    ///
    /// NaN values aren't equal to themselves, just like in IEEE 754. To check for NaN values, use either `is_nan` or, if you're using the `try_nan` feature (which currently only works on Nightly), the `Try` trait, which returns an error if the value is NaN.
    ///
    /// No following exponent or coefficients are allowed.
    pub const NAN: Self = Self(0b10_000000);

    /// Retreives the sign from a Head Byte.
    #[inline(always)]
    #[must_use]
    pub fn sign(self) -> Sign {
        Sign::from((self.into_inner() & Self::SIGN_MASK) != 0)
    }
    /// Retreives the absolute value from the number whose Head Byte is `self`.
    ///
    /// Since the Head Byte stores the sign of the entire number, it's enough to just perform bitwise `AND` with the `ABS_MASK`, which in turn is the bitwise `NOT` of the sign mask.
    #[inline(always)]
    #[must_use]
    pub fn abs(self) -> Self {
        Self(self.0 & Self::ABS_MASK)
    }
    /// Checks whether the `HAS_EXPONENT` bit of the Head Byte is set, meaning either infinity or the presence of an actual exponent.
    ///
    /// For a version which also checks for the infinity special case, see [`has_exponent`][0].
    ///
    /// [0]: #method.has_exponent "has_exponent — checks whether the Head Byte is supposed to be followed by an exponent byte"
    #[inline(always)]
    #[must_use]
    pub fn exponent_bit_set(self) -> bool {
        (self.0 & Self::HAS_EXPONENT_MASK) != 0
    }
    /// Checks whether the Head Byte describes either positive or negative infinity.
    ///
    /// This is mostly uesd in [`has_exponent`][0] to check for the infinity special case.
    ///
    /// [0]: #method.has_exponent "has_exponent — checks whether the Head Byte is supposed to be followed by an exponent byte"
    #[inline(always)]
    #[must_use]
    pub fn is_infinite(self) -> bool {
        self.abs() == Self::INFINITY
    }
    /// Checks whether the Head Byte describes a NaN value.
    #[inline(always)]
    #[must_use]
    pub fn is_nan(self) -> bool {
        (self.0 & Self::SIGN_MASK) != 0 && (self.0 & Self::ABS_MASK) == 0
    }
    /// Checks whether the Head Byte is supposed to be followed by an exponent byte.
    ///
    /// This includes the check for the special infinity value. For a version which does not check for infinity and thus plays more nicely with branch prediction, see [`exponent_bit_set`][0].
    ///
    /// [0]: #method.exponent_bit_set "exponent_bit_set — checks whether the HAS_EXPONENT bit of the Head Byte is set, meaning either infinity or the presence of an actual exponent"
    #[inline(always)]
    #[must_use]
    pub fn has_exponent(self) -> bool {
        ( (self.0 & Self::HAS_EXPONENT_MASK) != 0 ) && ( !self.is_infinite() )
    }
    /// Retreives the number of following coefficients from a Head Byte.
    #[inline(always)]
    #[must_use]
    pub fn num_coefficients(self) -> u8 {
        self.0 & Self::NUM_COEFFICIENTS_MASK
    }

    /// Consumes the value and returns the inner byte.
    #[inline(always)]
    #[must_use]
    pub fn into_inner(self) -> u8 {
        self.0
    }
}
impl From<u8> for HeadByte {
    /// Wraps a byte into a Head Byte.
    #[inline(always)]
    #[must_use]
    fn from(op: u8) -> Self {
        Self(op)
    }
}
impl From<HeadByte> for u8 {
    /// Consumes the Head Byte and returns the underlying inner byte.
    #[inline(always)]
    #[must_use]
    fn from(op: HeadByte) -> Self {
        op.0
    }
}
impl core::ops::Neg for HeadByte {
    type Output = Self;
    #[inline(always)]
    #[must_use]
    fn neg(self) -> Self::Output {
        if self.is_nan() || self == Self::ZERO {
            self
        } else {
            Self(!(self.0 & Self::SIGN_MASK) | self.abs().0)
        }
    }
}
impl core::fmt::Debug for HeadByte {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        if self.is_nan() {
            let mut ds = fmt.debug_tuple("HeadByte");
            ds.field(&"NaN");
            ds.finish()
        } else if self.is_infinite() {
            let mut ds = fmt.debug_tuple("HeadByte");
            ds.field(&match self.sign() {
                Sign::Positive => "Infinity",
                Sign::Negative => "-Infinity"
            });
            ds.finish()
        } else if *self == Self::ZERO {
            let mut ds = fmt.debug_tuple("HeadByte");
            ds.field(&"0");
            ds.finish()
        } else {
            let mut ds = fmt.debug_struct("HeadByte");
            ds.field("sign", &crate::SignDisplayAsDebug(self.sign()));
            ds.field("has_exponent", &self.has_exponent());
            ds.field("num_coefficients", &self.num_coefficients());
            ds.finish()
        }
    }
}

/// An exponent for the Head Byte and Extended Head Byte formats.
///
/// To retreive the real value of an \[E\]HB number, its stored value is multiplied by 10 raised to the power of this value as retreived using [`into_inner`][ii].
///
/// [ii]: #method.into_inner "into_inner — consumes the value and returns the inner byte"
///
/// This is **not** a 2's complement signed number: it ranges from -127 to +127, having one bit as the sign and the rest as a normal 7-bit unsigned integer. As a consequence, it's possible to store `0b1_0000000` as the exponent, meaning a resulting exponent of 10⁻⁰, which is undefined. In most cases, this transformation is unwanted (that is, accidential, most likely happening because of a serious mistake during bitwise operations), and as such is not allowed, producing a `TryFrom` error.
///
/// In other words, **protection against `-0` is a safety guarantee**, and actually creating an exponent with this value **requires unsafe code**.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Exponent(u8);

impl Exponent {
    /// The sign bit mask.
    ///
    /// Use [`sign`][0] to easily retreive the sign.
    ///
    /// [0]: #method.sign "sign — retreives the sign of the exponent"
    pub const SIGN_MASK: u8 = 0b1000_0000;
    pub const ABS_MASK: u8 = !Self::SIGN_MASK;

    /// Wraps a byte into an exponent, ignoring the invalid `0b1_0000000` case.
    ///
    /// This is the unsafe unchecked version of the `TryFrom` implementation.
    ///
    /// # Safety
    /// The value must never be `0b1_0000000` (`-0`), since avoiding that case is a safety guaranteee of the `Exponent` type.
    #[inline(always)]
    pub unsafe fn from_u8_unchecked(op: u8) -> Self {
        Self(op)
    }
    /// Retreives the sign from the exponent. `Negative` means that the coefficient is multiplied by 10 raised to the power of `-n`, where `n` is the rest of the exponent byte, and `Positive` simply means `10^n`.
    #[inline(always)]
    pub fn sign(self) -> Sign {
        Sign::from((self.0 & Self::SIGN_MASK) != 0)
    }
    /// Retreives the absolute value of the exponent, i.e. removes the minus in `10^-n` if it exists.
    #[inline(always)]
    #[must_use = "this is not an in-place operation"]
    pub fn abs(self) -> Self {
        Self(self.0 & Self::ABS_MASK)
    }
    /// Inverts the sign bit of the exponent. `10^2` (`0b0000_0010`) becomes `10^-2` (`0b1000_0010`), `10^127` → `10^-127` and so on.
    #[inline(always)]
    #[must_use = "this is not an in-place operation"]
    pub fn invert(self) -> Self {
        let sign = !(self.0 & Self::SIGN_MASK);
        Self((self.0 & Self::ABS_MASK) | sign)
    }

    #[inline(always)]
    #[must_use = "this is not an in-place operation"]
    pub fn checked_mul(self, rhs: Self) -> Option<Self> {
        if rhs.sign() == self.sign() {
            let sign = self.0 & Self::SIGN_MASK;
            if let Some(result) = self.abs().0.checked_add(rhs.abs().0) {
                Some(Self(sign | result))
            } else {None}

        } else {
            self.checked_div(rhs)
        }
    }
    #[inline(always)]
    #[must_use = "this is not an in-place operation"]
    fn checked_div(self, rhs: Self) -> Option<Self> {
        if rhs.sign() == self.sign() {
            let sign = self.0 & Self::SIGN_MASK;
            if let Some(result) = self.abs().0.checked_sub(rhs.abs().0) {
                Some(Self(sign | result))
            } else {None}
        } else {
            self.checked_mul(rhs)
        }
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