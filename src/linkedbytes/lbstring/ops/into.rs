use alloc::string::String;
use crate::LBString;

impl From<LBString> for String {
    #[inline(always)]
    fn from(op: LBString) -> Self {
        op.chars().collect::<Self>()
    }
}