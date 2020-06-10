use crate::LBString;
use alloc::string::String;
use core::cmp::Ordering; // We're not gonna deal with atomics here, right?

impl PartialEq for LBString {
    #[inline(always)]
    fn eq(&self, rhs: &Self) -> bool {
        self.cmp(rhs) == Ordering::Equal
    }
}
impl Eq for LBString {}
impl PartialOrd for LBString {
    #[inline(always)]
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}
impl Ord for LBString {
    #[inline]
    fn cmp(&self, rhs: &Self) -> Ordering {
        let (mut chars_l, mut chars_r) = (self.chars(), rhs.chars());
        loop {
            let (l, r) = (chars_l.next(), chars_r.next());
            if let Some(l_val) = l {
                if let Some(r_val) = r {
                    match l_val.cmp(&r_val) {
                        Ordering::Greater => {return Ordering::Greater;},
                        Ordering::Less => {return Ordering::Less;},
                        Ordering::Equal => {}
                    }
                } else {return Ordering::Greater;}
            } else if r.is_some() {return Ordering::Less;} else {break;}
        }
        Ordering::Equal
    }
}

macro_rules! impl_pcmp_for_chars {
    ($ty:ident) => {
        impl PartialEq<$ty> for LBString {
            #[inline(always)]
            fn eq(&self, rhs: &$ty) -> bool {
                self.partial_cmp(rhs) == Some(Ordering::Equal)
            }
        }
        impl PartialOrd<$ty> for LBString {
            fn partial_cmp(&self, rhs: &$ty) -> Option<Ordering> {
                let (mut chars_l, mut chars_r) = (self.chars(), rhs.chars());
                loop {
                    let (l, r) = (chars_l.next(), chars_r.next());
                    if let Some(l_val) = l {
                        if let Some(r_val) = r {
                            match l_val.cmp(&r_val) {
                                Ordering::Greater => {return Some(Ordering::Greater);},
                                Ordering::Less => {return Some(Ordering::Less);},
                                Ordering::Equal => {}
                            }
                        } else {return Some(Ordering::Greater);}
                    } else if r.is_some() {return Some(Ordering::Less);} else {break;}
                }
                Some(Ordering::Equal)
            }
        }

        impl PartialEq<LBString> for $ty {
            #[inline(always)]
            fn eq(&self, rhs: &LBString) -> bool {
                self.partial_cmp(rhs) == Some(Ordering::Equal)
            }
        }
        impl PartialOrd<LBString> for $ty {
            #[inline(always)]
            fn partial_cmp(&self, rhs: &LBString) -> Option<Ordering> {
                rhs.partial_cmp(self)
            }
        }
    };
}

impl_pcmp_for_chars!(String);
impl_pcmp_for_chars!(str);