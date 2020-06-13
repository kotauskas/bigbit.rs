use crate::{
    Gcd,
    linkedbytes::*,
};

impl Gcd for LBNum {
    /// Uses the Euclidean algorithm to calculate the GCD of two numbers encoded using Linked Bytes.
    ///
    /// # Usage
    /// ```
    /// # use bigbit::prelude::*;
    /// let x = LBNum::from(18_u8);
    /// let y = LBNum::from(12_u8);
    /// let gcd = bigbit::gcd(x, y);
    /// // The result is 6:
    /// println!("The GCD of 12 and 18 is {}", gcd);
    /// # assert_eq!(gcd, 6_u8);
    /// ```
    fn gcd(mut lhs: Self, mut rhs: Self) -> Self {
        loop {
            match lhs.cmp(&rhs) {
                Ordering::Greater => {
                    // lhs is greater than rhs
                    dbg!(&lhs);
                    dbg!(&rhs);
                    lhs -= &rhs;
                },
                Ordering::Less => {
                    // rhs is greater than lhs
                    dbg!(&lhs);
                    dbg!(&rhs);
                    rhs -= &lhs;
                },
                Ordering::Equal => break,
            };
        }
        // It obviously doesn't matter which one we return, since we only get here when both are equal.
        lhs
    }
}