use crate::{LBNum, linkedbytes::LBSequence, LinkedByte};
use core::convert::TryFrom;
use alloc::vec;

/// This tests the overflowing behavior as well as the general addition capabilities.
#[test]
fn lb_add() {
    let mut num = LBNum::try_from(
        LBSequence::from(vec![
            LinkedByte::from(41).into_linked(),
            LinkedByte::from(127),
    ])).unwrap();

    num += 87_u8; // haha funny bite number

    let expected = LBNum::try_from(
        LBSequence::from(vec![
            LinkedByte::from(0).into_linked(),
            LinkedByte::from(0).into_linked(),
            LinkedByte::from(1)
    ])).unwrap();
    assert_eq!(num, expected);
}