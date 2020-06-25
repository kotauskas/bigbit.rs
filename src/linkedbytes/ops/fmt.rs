use crate::{
    LBNum, RADIX_TABLE, DivRemAssign,
};
use core::{
    fmt::{self, Formatter},
    hint,
    convert::TryInto,
};
use alloc::string::{
    String, ToString,
};

impl LBNum {
    /// Converts a Linked Bytes number into a string with an arbitrary radix (base), from 2 to 36 inclusively.
    ///
    /// All alphabetic characters in the result are uppercase — use [`make_ascii_lowercase`][mal] if you want `LowerHex`-style formatting.
    ///
    /// [mal]: https://doc.rust-lang.org/std/primitive.str.html#method.make_ascii_lowercase "make_ascii_lowercase — converts this string to its ASCII lower case equivalent in-place"
    ///
    /// # Panics
    /// Passing a radix less than 2 or greater than 36 results an immediate panic, even if the value is 0.
    pub fn into_string_with_radix(mut self, radix: u8) -> String {
        if radix < 2 || radix > 36 { // Make sure that the radix is valid.
            panic!("invalid value for radix (not in range from 2 to 36, inclusively)");
        }
        if self == 0_u8 {return '0'.to_string();} // Avoid an empty string condition.
        let mut result = String::new();
        while self != 0_u8 {
            let remainder = self.div_rem_assign(radix);
            let remainder: usize = (&remainder).try_into().unwrap_or_else(|_| unsafe {hint::unreachable_unchecked()});
            result.push(RADIX_TABLE[remainder]);
        }
        // Since all of our graphemes are ASCII, there's no way
        // that any combining characters can ever get in the
        // way when reversing the result. We can simply get an
        // iterator over the codepoints, reverse it and collect
        // it into the same variable name.
        let result: String = result.chars().rev().collect();
        result
    }
}

impl fmt::Display for LBNum {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(&self.clone().into_string_with_radix(10))
    }
}
impl fmt::Binary for LBNum {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(&self.clone().into_string_with_radix(2))
    }
}
impl fmt::Octal for LBNum {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(&self.clone().into_string_with_radix(8))
    }
}
impl fmt::UpperHex for LBNum {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(&self.clone().into_string_with_radix(16))
    }
}
impl fmt::LowerHex for LBNum {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut stringified = self.clone().into_string_with_radix(16);
        stringified.make_ascii_lowercase();
        f.write_str(&stringified)
    }
}