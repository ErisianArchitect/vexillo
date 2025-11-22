use ::core::cmp::Ordering;
use ::core::ops::Range;
pub use vexmacro::const_binary_search_fn;
#[doc(hidden)]
pub use vexproc::flags;

mod private {
    pub trait Sealed {}
}

pub trait MustBeUnsignedInt: private::Sealed + Clone + Copy {}

macro_rules! mark_types {
    ($($type:ty),+$(,)?) => {
        $(
            impl private::Sealed for $type {}
            impl MustBeUnsignedInt for $type {}
        )*
    };
}

mark_types!(
    u8,
    u16,
    u32,
    u64,
    // TODO: It might be a good idea to exclude u128, but I can't remember why.
    u128,
    // usize is excluded due to variable width.
    // Variable width breaks serialization between targets.
    // usize,
);

pub const fn mask_type_check<MaskType: MustBeUnsignedInt>() {}

#[doc(hidden)]
#[macro_export]
macro_rules! mask_type_check {
    ($type:ty) => {
        const _: () = { $crate::internal::mask_type_check::<$type>(); };
    };
}

pub const fn const_cmp_str(lhs: &str, rhs: &str) -> Ordering {
    let min_len = if lhs.len() <= rhs.len() {
        lhs.len()
    } else {
        rhs.len()
    };
    let mut byte_index = 0usize;
    let lhs = lhs.as_bytes();
    let rhs = rhs.as_bytes();
    while byte_index < min_len {
        if lhs[byte_index] < rhs[byte_index] {
            return Ordering::Less;
        } else if lhs[byte_index] > rhs[byte_index] {
            return Ordering::Greater;
        }
        byte_index += 1;
    }
    if lhs.len() == rhs.len() {
        Ordering::Equal
    } else if lhs.len() <= rhs.len() {
        Ordering::Less
    } else {
        Ordering::Greater
    }
}

pub struct ConstCounter<T> {
    pub count: T,
}

macro_rules! counter_impls {
    ($($count_ty:ty),*$(,)?) => {
        $(
            impl ConstCounter<$count_ty> {
                #[must_use]
                #[inline(always)]
                pub const fn next(&mut self) -> $count_ty {
                    let next = self.count;
                    self.count += 1;
                    next
                }
                
                #[must_use]
                #[inline(always)]
                pub const fn incr(&mut self) {
                    _=self.next();
                }
            }
        )*
    };
}

impl<T> ConstCounter<T> {
    #[must_use]
    #[inline(always)]
    pub const fn new(start: T) -> Self {
        Self {
            count: start,
        }
    }
}

counter_impls!(
    u8,
    u16,
    u32,
    u64,
    usize,
);

#[derive(Clone, Copy)]
pub struct MaskIndex {
    pub mask: usize,
    pub bit: u32,
}

impl MaskIndex {
    #[must_use]
    #[inline(always)]
    pub const fn new(index: u32, bit_size: u32) -> Self {
        Self {
            mask: (index / bit_size) as usize,
            bit: index % bit_size,
        }
    }
}

#[must_use]
#[track_caller]
#[inline(always)]
pub const fn subslice<T>(slice: &[T], range: Range<usize>) -> &[T] {
    assert!(
        range.end <= slice.len() && range.start <= range.end,
        "Range out of bounds."
    );
    unsafe {
        ::core::slice::from_raw_parts(
            slice.as_ptr().add(range.start),
            range.end - range.start
        )
    }
}

#[must_use]
#[track_caller]
#[inline(always)]
pub const unsafe fn subslice_mut<'a, T>(ptr: *mut T, range: Range<usize>) -> &'a mut [T] {
    assert!(
        range.start <= range.end,
        "Range out of bounds."
    );
    unsafe {
        ::core::slice::from_raw_parts_mut(
            ptr,
            range.end - range.start
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn subslice_mut_test() {
        let mut bytes = [0u8; 32];
        let bptr = bytes.as_mut_ptr();
        let sub = unsafe { subslice_mut(bptr, 0..4) };
        sub.iter_mut().for_each(|b| *b = 67);
        println!("{:?}", bytes);
    }
}