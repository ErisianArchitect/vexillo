use std::cmp::Ordering;

mod private {
    pub trait Sealed {}
}

#[doc(hidden)]
pub trait MustBeUnsignedInt: private::Sealed {}

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

#[doc(hidden)]
pub const fn mask_type_check<MaskType: MustBeUnsignedInt>() {}

#[doc(hidden)]
#[macro_export]
macro_rules! mask_type_check {
    ($type:ty) => {
        const _: () = { $crate::mask_type_check::<$type>(); };
    };
}

#[doc(hidden)]
pub const fn const_cmp_str(lhs: &str, rhs: &str) -> Ordering {
    let (min_len, len_cmp) = if lhs.len() <= rhs.len() {
        (
            lhs.len(),
            std::cmp::Ordering::Less,
        )
    } else {
        (
            rhs.len(),
            std::cmp::Ordering::Greater,
        )
    };
    let mut byte_index = 0usize;
    let lhs = lhs.as_bytes();
    let rhs = rhs.as_bytes();
    while byte_index < min_len {
        if lhs[byte_index] < rhs[byte_index] {
            return std::cmp::Ordering::Less;
        } else if lhs[byte_index] > rhs[byte_index] {
            return std::cmp::Ordering::Greater;
        }
        byte_index += 1;
    }
    if lhs.len() == rhs.len() {
        std::cmp::Ordering::Equal
    } else {
        len_cmp
    }
}

#[doc(hidden)]
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
                    self.next();
                }
            }
        )*
    };
}

impl<T> ConstCounter<T> {
    #[inline(always)]
    #[must_use]
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

#[doc(hidden)]
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

#[doc(hidden)]
#[must_use]
#[inline(always)]
pub const fn subslice<T>(slice: &[T], start: usize, len: usize) -> &[T] {
    unsafe {
        core::slice::from_raw_parts(slice.as_ptr().add(start), len)
    }
}

#[doc(hidden)]
#[must_use]
#[inline(always)]
pub const fn subslice_mut<T>(slice: &mut [T], start: usize, len: usize) -> &mut [T] {
    unsafe {
        core::slice::from_raw_parts_mut(slice.as_mut_ptr().add(start), len)
    }
}