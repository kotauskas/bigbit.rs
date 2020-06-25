use crate::LBNum;

macro_rules! impl_from_primitive {
    ($($ty:ident)+) => ($(
        impl From<$ty> for LBNum {
            #[inline(always)]
            fn from(op: $ty) -> Self {
                Self::ZERO + op
            }
        }
    )+)
}

impl_from_primitive! {
    u8 u16 u32 u64 u128 usize
}