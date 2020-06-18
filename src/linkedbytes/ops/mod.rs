use crate::linkedbytes::{LBNum, LBNumRef};
use core::cmp::{self, Ordering};

mod add; mod sub; mod mul; mod div; mod from; mod tryinto; mod fmt; mod gcd;
pub(crate) use sub::DecrementResult;

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
    #[inline(always)]
    fn cmp(&self, rhs: &Self) -> Ordering {
        LBNumRef::from(self).cmp(&LBNumRef::from(rhs))
    }
}
impl PartialEq for LBNumRef<'_> {
    #[inline(always)]
    fn eq(&self, rhs: &Self) -> bool {
        self.cmp(rhs) == Ordering::Equal
    }
}
impl Eq for LBNumRef<'_> {}
impl cmp::PartialOrd for LBNumRef<'_> {
    #[inline(always)]
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}
impl<'a> cmp::Ord for LBNumRef<'a> {
    #[inline]
    fn cmp(&self, rhs: &Self) -> Ordering {
        match self.inner().len().cmp(&rhs.inner().len()) {
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
impl PartialEq<LBNumRef<'_>> for LBNum {
    #[inline(always)]
    fn eq(&self, rhs: &LBNumRef<'_>) -> bool {
        LBNumRef::from(self).cmp(rhs) == Ordering::Equal
    }
}
impl PartialOrd<LBNumRef<'_>> for LBNum {
    #[inline(always)]
    fn partial_cmp(&self, rhs: &LBNumRef<'_>) -> Option<Ordering> {
        Some(LBNumRef::from(self).cmp(rhs))
    }
}
impl PartialEq<LBNum> for LBNumRef<'_> {
    #[inline(always)]
    fn eq(&self, rhs: &LBNum) -> bool {
        self.cmp(&LBNumRef::from(rhs)) == Ordering::Equal
    }
}
impl PartialOrd<LBNum> for LBNumRef<'_> {
    #[inline(always)]
    fn partial_cmp(&self, rhs: &LBNum) -> Option<Ordering> {
        Some(self.cmp(&LBNumRef::from(rhs)))
    }
}

macro_rules! impl_partial_eq_ord_to_primitive {
    ($ty:ident) => {
        impl PartialEq<$ty> for LBNum {
            #[inline(always)]
            fn eq(&self, rhs: &$ty) -> bool {
                *self == Self::from(*rhs)
            }
        }
        impl PartialEq<$ty> for LBNumRef<'_> {
            #[inline(always)]
            fn eq(&self, rhs:&$ty) -> bool {
                *self == LBNum::from(*rhs)
            }
        }
        impl PartialEq<LBNum> for $ty {
            #[inline(always)]
            fn eq(&self, rhs: &LBNum) -> bool {
                LBNum::from(*self) == *rhs
            }
        }
        impl PartialEq<LBNumRef<'_>> for $ty {
            #[inline(always)]
            fn eq(&self, rhs: &LBNumRef<'_>) -> bool {
                LBNum::from(*self) == *rhs
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
        impl PartialOrd<$ty> for LBNumRef<'_> {
            /// Compares `self` and `rhs`.
            ///
            /// Never fails, a return value of `Some` can be relied upon.
            #[inline(always)]
            fn partial_cmp(&self, rhs: &$ty) -> Option<Ordering> {
                self.partial_cmp(&LBNum::from(*rhs)) // Why doesn't Ord have a type parameter?
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
        impl PartialOrd<LBNumRef<'_>> for $ty {
            /// Compares `self` and `rhs`.
            ///
            /// Never fails, a return value of `Some` can be relied upon.
            #[inline(always)]
            fn partial_cmp(&self, rhs: &LBNumRef<'_>) -> Option<Ordering> {
                LBNum::from(*self).partial_cmp(rhs)
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