use crate::{
    linkedbytes::{LBNum, LBNumRef},
    POWERS_OF_128,
};
use core::{
    convert::TryFrom,
    fmt::{self, Formatter, Display},
};

macro_rules! impl_from_lb_for_primitive {
    ($($ty:ident)+) => ($(
        impl<'r> TryFrom<LBNumRef<'r>> for $ty {
            type Error = TryFromIntError;

            fn try_from(op: LBNumRef<'r>) -> Result<Self, TryFromIntError> {
                if op.inner().len() > POWERS_OF_128.len() {return Err(TryFromIntError);}
                let mut result: $ty = 0;
                for (num, el) in op.inner().iter().enumerate() {
                    let tbl_power = POWERS_OF_128[num];
                    if let Ok(power) = $ty::try_from(tbl_power) {
                        if let Some(val) = $ty::from(el.into_int7()).checked_mul(power) {
                            if let Some(added) = result.checked_add(val) {
                                result = added;
                            } else {return Err(TryFromIntError);}
                        }
                    } else {return Err(TryFromIntError);}

                }
                Ok(result)
            }
        }
        impl TryFrom<&LBNum> for $ty {
            type Error = TryFromIntError;

            #[inline(always)]
            fn try_from(op: &LBNum) -> Result<$ty, TryFromIntError> {
                $ty::try_from(LBNumRef::from(op))
            }
        }

        impl TryFrom<LBNum> for $ty {
            type Error = TryFromIntError;

            #[inline(always)]
            fn try_from(op: LBNum) -> Result<$ty, TryFromIntError> {
                $ty::try_from(&op)
            }
        }
    )+)
}

/// Marker error type indicating that an integer conversion from a Linked Bytes number has failed.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct TryFromIntError;
impl Display for TryFromIntError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("conversion from Linked Bytes to scalar integer failed")
    }
}
#[cfg(feature = "std")]
impl std::error::Error for TryFromIntError {}

impl_from_lb_for_primitive! {
    // FIXME: Because LinkedByte stores a u8, converting into i8 currently complains. i8::try_from(u8::try_from(...).unwrap()).unwrap() works though.
    u8
    u16     i16
    u32     i32
    u64     i64
    u128   i128
    usize isize
}