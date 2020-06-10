use crate::{LBNum, linkedbytes::LBNumRef, POWERS_OF_128};
use core::{convert::TryFrom};

macro_rules! impl_from_lb_for_primitive {
    ($ty:ident) => {
        impl<'a> TryFrom<LBNumRef<'a>> for $ty {
            type Error = TryFromIntError;

            fn try_from(op: LBNumRef<'a>) -> Result<Self, TryFromIntError> {
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
        impl<'a> TryFrom<&'a LBNum> for $ty {
            type Error = TryFromIntError;

            #[inline(always)]
            fn try_from(op: &'a LBNum) -> Result<$ty, TryFromIntError> {
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
    };
}

/// Marker error type indicating that an integer
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct TryFromIntError;

impl_from_lb_for_primitive!(u8  );
// FIXME: Because LinkedByte stores a u8, converting into i8 currently complains. Currently i8::try_from(u8::try_from(...).unwrap()).unwrap() works.
// impl_from_lb_for_primitive!(i8  );
impl_from_lb_for_primitive!(u16  );
impl_from_lb_for_primitive!(i16  );
impl_from_lb_for_primitive!(u32  );
impl_from_lb_for_primitive!(i32  );
impl_from_lb_for_primitive!(u64  );
impl_from_lb_for_primitive!(i64  );
impl_from_lb_for_primitive!(u128 );
impl_from_lb_for_primitive!(i128 );
impl_from_lb_for_primitive!(usize);
impl_from_lb_for_primitive!(isize);