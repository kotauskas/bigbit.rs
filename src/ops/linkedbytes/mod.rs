use crate::{LBNum};
use core::cmp::{self, Ordering};

mod add; mod sub; mod mul; mod div; mod from; mod tryinto; mod fmt;
#[allow(unused_imports)]
pub(crate) use {add::*, sub::*, mul::*, div::*, from::*, tryinto::*, fmt::*};

impl cmp::PartialEq for LBNum {
    #[inline(always)]
    fn eq(&self, rhs: &Self) -> bool {
        self.cmp(rhs) == Ordering::Equal
    }
}
impl cmp::Eq for LBNum {}
impl cmp::PartialOrd for LBNum {
    #[inline(always)]
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}
impl cmp::Ord for LBNum {
    #[inline]
    fn cmp(&self, rhs: &Self) -> Ordering {
        match self.0.len().cmp(&rhs.0.len()) {
            Ordering::Greater => Ordering::Greater,
            Ordering::Less => Ordering::Less,
            Ordering::Equal => {
                for (this, other) in self.iter_be().zip(rhs.iter_be()) {
                    match this.cmp(&other) {
                        Ordering::Greater => {return Ordering::Greater},
                        Ordering::Less => {return Ordering::Less},
                        Ordering::Equal => {} // Do nothing in this case, search for the next one.
                    }
                }
                Ordering::Equal
            }
        }
    }
}

macro_rules! impl_partial_eq_ord_to_primitive {
    ($ty:ident) => {
        impl PartialEq<$ty> for LBNum {
            #[inline(always)]
            fn eq(&self, rhs: &$ty) -> bool {
                self == &Self::from(*rhs)
            }
        }
        impl PartialEq<LBNum> for $ty {
            #[inline(always)]
            fn eq(&self, rhs: &LBNum) -> bool {
                &LBNum::from(*self) == rhs
            }
        }
        impl PartialOrd<$ty> for LBNum {
            /// Compares `self` and `rhs`.
            ///
            /// Never fails, a return value of `Some` can be relied upon.
            #[inline(always)]
            fn partial_cmp(&self, rhs: &$ty) -> Option<Ordering> {
                Some(self.cmp(&Self::from(*rhs)))
            }
        }
        impl PartialOrd<LBNum> for $ty {
            /// Compares `self` and `rhs`.
            ///
            /// Never fails, a return value of `Some` can be relied upon.
            #[inline(always)]
            fn partial_cmp(&self, rhs: &LBNum) -> Option<Ordering> {
                Some(LBNum::from(*self).cmp(rhs))
            }
        }
    };
}

impl_partial_eq_ord_to_primitive!(u8   );
impl_partial_eq_ord_to_primitive!(u16  );
impl_partial_eq_ord_to_primitive!(u32  );
impl_partial_eq_ord_to_primitive!(u64  );
impl_partial_eq_ord_to_primitive!(u128 );
impl_partial_eq_ord_to_primitive!(usize);