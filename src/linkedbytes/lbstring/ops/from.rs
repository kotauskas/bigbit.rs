use alloc::string::String;
use crate::LBString;

impl From<&String> for LBString {
    #[inline(always)]
    fn from(op: &String) -> Self {
        op.chars().collect::<Self>()
    }
}
impl From<String> for LBString {
    #[inline(always)]
    fn from(op: String) -> Self {
        op.chars().collect::<Self>()
    }
}
impl<'a> From<&'a str> for LBString {
    #[inline(always)]
    fn from(op: &'a str) -> Self {
        op.chars().collect::<Self>()
    }
}