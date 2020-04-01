use crate::LBNum;

macro_rules! impl_from_primitive {
    ($ty:ident) => {
        impl From<$ty> for LBNum {
            #[inline(always)]
            fn from(op: $ty) -> Self {
                Self::ZERO + op
            }
        }
    };
}

impl_from_primitive!(u8   );
impl_from_primitive!(u16  );
impl_from_primitive!(u32  );
impl_from_primitive!(u64  );
impl_from_primitive!(u128 );
impl_from_primitive!(usize);