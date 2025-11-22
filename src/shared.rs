use ::core::fmt::Debug;
use ::core::hash::Hash;
use ::core::ops::{
    Not,
    BitAnd, BitAndAssign,
    BitOr, BitOrAssign,
    BitXor, BitXorAssign,
    Add, AddAssign,
    Sub, SubAssign,
    Index,
    Range,
};
use vexmacro::const_binary_search_fn;

pub trait Flags: 'static
    + Sized
    + Send
    + Sync
    + Debug
    + Copy
    + Eq
    + Ord
    + Hash
    // Bitwise
    + Not
    + BitAnd<Self, Output = Self>
    + BitAndAssign<Self>
    + BitOr<Self, Output = Self>
    + BitOrAssign<Self>
    + BitXor<Self, Output = Self>
    + BitXorAssign<Self>
    + Add<Self, Output = Self>
    + AddAssign<Self>
    + Sub<Self, Output = Self>
    + SubAssign<Self>
    + Index<u32, Output = bool>
    + Index<usize, Output = bool>
{
    /// The type that is used for the internal bitmasks.
    type MaskType;
    type MasksArrayType;
    type BytesArrayType;
    
    /// The total number of bits for this type. This is equal to `size_of::<Self> * 8`.
    const BITS: u32;
    const USED_BITS: u32;
    const UNUSED_BITS: u32;
    /// The total number of bits for the mask type.
    const MASK_BITS: u32;
    /// The total number of bytes for the mask type.
    const MASK_SIZE: usize;
    /// The total number of masks in the internal array.
    const MASK_COUNT: usize;
    
    /// The total number of single-bit flags.
    const SINGLE_FLAG_COUNT: usize;
    /// The total number of union flags (flags composed of multiple other flags).
    const GROUP_FLAG_COUNT: usize;
    /// The total number of single and group flags.
    const TOTAL_FLAG_COUNT: usize;
    /// An instance with none of the bits set to 1.
    const NONE: Self;
    /// An instance with all of the bits set to 1.
    const ALL: Self;
    
    /// A table of all flags ordered first by [single, group], then ordered by declaration order.
    /// 
    /// - `FLAGS_TABLE[..SINGLE_FLAG_COUNT]` are the single flags.
    /// - `FLAGS_TALE[SINGLE_FLAG_COUNT..]` are the group flags.
    const FLAGS_TABLE: &'static [FlagRow<Self>];
    
    const SINGLE_FLAGS: &'static [FlagRow<Self>];
    const GROUP_FLAGS: &'static [FlagRow<Self>];
    const BIT_ORDERED_FLAGS_TABLE: &'static [FlagIndex];
    const ORDERED_FLAGS_TABLE: &'static [FlagIndex];
    const ORDERED_SINGLE_FLAG_INDICES: &'static [FlagIndex];
    const ORDERED_GROUP_FLAG_INDICES: &'static [FlagIndex];
    
    /// Create a new instance with none of the bits set.
    fn new() -> Self;
    /// Create a new instance with none of the bits set.
    fn none() -> Self;
    /// Create a new instance with all of the bits set.
    fn all() -> Self;
    /// Create a union of all `flags`.
    fn union(flags: &[Self]) -> Self;
    /// Create a union of all `flags` without all `removals`.
    fn union_without(flags: &[Self], removals: &[Self]) -> Self;
    /// Try to find a flag by its name. Returns [None] if the flag was not found.
    fn try_find(name: &str) -> Option<Self>;
    /// Find a flag by its name. Panics if the flag was not found.
    fn find(name: &str) -> Self;
    /// Find a flag by its name. Returns `default` if the flag was not found.
    fn find_or(name: &str, default: Self) -> Self;
    /// Find a flag by its name. Returns `NONE` if the flag was not found.
    fn find_or_none(name: &str) -> Self;
    /// Return the number of ones in the binary representation of `self`.
    fn count_ones(self) -> u32;
    /// Return the number of zeros in the binary representation of `self`.
    fn count_zeros(self) -> u32;
    /// Get the bit at `index`.
    fn get(self, index: u32) -> bool;
    /// Set the bit at `index`.
    fn set(&mut self, index: u32, on: bool) -> &mut Self;
    /// Swap the bit at `index`.
    fn swap(&mut self, index: u32, on: bool) -> bool;
    /// Create an instance with the bit at the given `inde` set to 1.
    fn from_index(index: u32) -> Self;
    /// Add all of the bits present in `flag`.
    fn add(&mut self, flag: Self) -> &mut Self;
    /// Add all of the bits present in all `flags`.
    fn add_all(&mut self, flags: &[Self]) -> &mut Self;
    /// Remove all of the bits present in `flag`.
    fn remove(&mut self, flag: Self) -> &mut Self;
    /// Remove all of the bits present in all `flags`.
    fn remove_all(&mut self, flags: &[Self]) -> &mut Self;
    /// Join with `flag`.
    fn with(self, flag: Self) -> Self;
    /// Join with all `flags`.
    fn with_all(self, flags: &[Self]) -> Self;
    /// Exclude `flag`.
    fn without(self, flag: Self) -> Self;
    /// Exclude all `flags`.
    fn without_all(self, flags: &[Self]) -> Self;
    /// Test if all bits of `flag` are present in `self`.
    fn has_all(self, flag: Self) -> bool;
    /// Test if none of the bits of `flag` are present in `self`.
    fn has_none(self, flag: Self) -> bool;
    /// Test if any of the bits of `flag` are present in `self`.
    fn has_any(self, flag: Self) -> bool;
    /// Returns the inner `masks` as a slice.
    fn as_slice(&self) -> &[Self::MaskType];
    /// Returns the inner `masks` as a mutable slice.
    fn as_mut_slice(&mut self) -> &mut [Self::MaskType];
    /// Decompose `self` into the inner `masks` array.
    fn into_inner(self) -> Self::MasksArrayType;
    /// Return `self` as a slice of bytes. Endianess is dependent on the target architecture. This is meant to be used for FFI. If you need serialization, use `to_be_bytes`, `to_le_bytes`, or `to_ne_bytes`.
    fn as_bytes(&self) -> &[u8];
    /// Return `self` as a mutable slice of bytes. Endianess is dependent on the target architecture. This is meant to be used for FFI. If you need serialization, use `to_be_bytes`, `to_le_bytes`, or `to_ne_bytes`.
    fn as_mut_bytes(&mut self) -> &mut [u8];
    /// Return `self` as Big-Endian bytes.
    fn to_be_bytes(self) -> Self::BytesArrayType;
    /// Create a new instance from Big-Endian `bytes`.
    fn from_be_bytes(bytes: Self::BytesArrayType) -> Self;
    /// Return `self` as Little-Endian bytes.
    fn to_le_bytes(self) -> Self::BytesArrayType;
    /// Create a new instance from Little-Endian `bytes`.
    fn from_le_bytes(bytes: Self::BytesArrayType) -> Self;
    /// Return `self` as Native-Endian bytes.
    fn to_ne_bytes(self) -> Self::BytesArrayType;
    /// Create a new instance from Native-Endian `bytes`.
    fn from_ne_bytes(bytes: Self::BytesArrayType) -> Self;
    /// Bitwise NOT.
    fn not(self) -> Self;
    /// Bitwise AND.
    fn and(self, other: Self) -> Self;
    /// Bitwise OR.
    fn or(self, other: Self) -> Self;
    /// Bitwise XOR.
    fn xor(self, other: Self) -> Self;
    /// Bitwise NAND.
    fn nand(self, other: Self) -> Self;
    /// Bitwise NOR.
    fn nor(self, other: Self) -> Self;
    /// Bitwise XNOR.
    fn xnor(self, other: Self) -> Self;
    /// Bitwise IMPLY.
    fn imply(self, other: Self) -> Self;
    /// Bitwise NIMPLY.
    fn nimply(self, other: Self) -> Self;
    /// Test for equality.
    fn eq(self, other: Self) -> Self;
    /// Test for inequality.
    fn ne(self, other: Self) -> Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FlagIndex {
    index: u16,
}

impl FlagIndex {
    #[must_use]
    #[inline(always)]
    pub const fn new(index: u16) -> Self {
        Self { index }
    }
    
    #[must_use]
    #[inline(always)]
    pub const fn index(self) -> usize {
        self.index as _
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FlagRow<T: Flags> {
    pub name: &'static str,
    pub value: T,
    sub_flag_indices: Option<&'static [FlagIndex]>,
}

impl<T: Flags> FlagRow<T> {
    #[must_use]
    #[inline(always)]
    pub const fn single(name: &'static str, value: T) -> Self {
        Self { name, value, sub_flag_indices: None }
    }
    
    #[must_use]
    #[inline(always)]
    pub const fn group(
        name: &'static str,
        value: T,
        sub_flag_indices: &'static [FlagIndex],
    ) -> Self {
        Self {
            name,
            value,
            sub_flag_indices: Some(sub_flag_indices),
        }
    }
    
    #[must_use]
    #[inline(always)]
    pub const fn bits(&self) -> u32 {
        if let Some(subflags) = self.sub_flag_indices {
            subflags.len() as u32
        } else {
            1
        }
    }
    
    #[must_use]
    #[inline(always)]
    pub const fn sub_flag_indices(&self) -> &'static [FlagIndex] {
        if let Some(subflags) = self.sub_flag_indices {
            subflags
        } else {
            &[]
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FlagGroupInfo {
    /// The bit count.
    pub bits: u32,
    /// The index in the flag table.
    pub index: u16,
}

#[derive(Clone)]
pub struct FlagTables<T: Flags, const TABLE_LEN: usize, const SINGLE_COUNT: usize, const GROUP_COUNT: usize> {
    pub rows: [FlagRow<T>; TABLE_LEN],
    pub name_ordered_row_indices: [FlagIndex; TABLE_LEN],
    pub name_ordered_single_indices: [FlagIndex; SINGLE_COUNT],
    pub name_ordered_group_indices: [FlagIndex; GROUP_COUNT],
    pub bit_ordered_group_indices: [FlagIndex; GROUP_COUNT],
}

#[derive(Clone, Copy)]
pub struct FlagRows<T: Flags> {
    pub rows: &'static [FlagRow<T>],
}

impl<T: Flags> FlagRows<T> {
    #[must_use]
    #[inline(always)]
    pub const fn new(rows: &'static [FlagRow<T>]) -> Self {
        Self { rows }
    }
    
    #[must_use]
    #[inline(always)]
    pub const fn rows(&self, range: Range<usize>) -> FlagRows<T> {
        FlagRows::new(crate::internal::subslice(self.rows, range))
    }
    
    #[must_use]
    #[inline(always)]
    pub const fn row(&self, index: usize) -> &'static FlagRow<T> {
        &self.rows[index]
    }
}

impl<
    T: Flags,
    const TABLE_LEN: usize,
    const SINGLE_COUNT: usize,
    const GROUP_COUNT: usize
> FlagTables<T, TABLE_LEN, SINGLE_COUNT, GROUP_COUNT> {
    #[must_use]
    #[inline(always)]
    pub const fn len() -> usize {
        TABLE_LEN
    }
    
    #[must_use]
    #[inline(always)]
    pub const fn single_len() -> usize {
        SINGLE_COUNT
    }
    
    #[must_use]
    #[inline(always)]
    pub const fn group_len() -> usize {
        GROUP_COUNT
    }
    
    #[must_use]
    #[inline(always)]
    pub const fn singles_range() -> Range<usize> {
        0..SINGLE_COUNT
    }
    
    #[must_use]
    #[inline(always)]
    pub const fn groups_range() -> Range<usize> {
        SINGLE_COUNT..TABLE_LEN
    }
    
    #[must_use]
    #[inline(always)]
    pub const fn row(&'static self, index: u16) -> &'static FlagRow<T> {
        &self.rows[index as usize]
    }
    
    #[must_use]
    #[inline(always)]
    pub const fn rows(&'static self, range: Range<usize>) -> FlagRows<T> {
        FlagRows::new(crate::internal::subslice(&self.rows, range))
    }
    
    #[must_use]
    #[inline(always)]
    pub const fn singles(&'static self) -> FlagRows<T> {
        self.rows(Self::singles_range())
    }
    
    #[must_use]
    #[inline(always)]
    pub const fn groups(&'static self) -> FlagRows<T> {
        self.rows(Self::groups_range())
    }
    
    #[must_use]
    #[inline(always)]
    pub const fn name(&'static self, index: u16) -> &'static str {
        self.row(index).name
    }
    
    #[must_use]
    #[inline(always)]
    pub const fn value(&'static self, index: u16) -> T {
        self.row(index).value
    }
    
    #[inline(always)]
    const fn rev_bit_count_search_cmp(lhs: u32, rhs_index: FlagIndex, context: &[FlagRow<T>]) -> ::core::cmp::Ordering {
        use ::core::cmp::Ordering::*;
        let rhs = context[rhs_index.index()].bits();
        if lhs > rhs {
            Less
        } else if lhs < rhs {
            Greater
        } else {
            Equal
        }
    }
    
    const_binary_search_fn!(
        use Self::rev_bit_count_search_cmp;
        #[must_use]
        #[inline(always)]
        const fn bit_count_binary_search(const u32, const FlagIndex, context: &[FlagRow<T>]) -> usize
    );
    
    /// Returns the first index of the row in `self.bit_ordered_group_indices` where `row.bits <= bits`.
    #[inline]
    #[must_use]
    #[track_caller]
    const fn upper_bit_count_search(&'static self, bits: u32, range: Range<usize>) -> usize {
        let start = range.start;
        Self::bit_count_binary_search(
            bits,
            crate::internal::subslice(&self.bit_ordered_group_indices, range),
            &self.rows
        ) + start
    }
    
    #[inline]
    #[must_use]
    #[track_caller]
    const fn upper_bit_count_search_full(&'static self, bits: u32) -> usize {
        self.upper_bit_count_search(
            bits,
            0..self.bit_ordered_group_indices.len()
        )
    }
}

#[derive(Clone)]
pub struct FlagConstants<
    T: Flags,
    const SINGLE_COUNT: usize,
    const GROUP_COUNT: usize,
    const TOTAL_COUNT: usize,
> {
    // Counts
    pub single_flag_count: usize,
    pub group_flag_count: usize,
    pub total_flag_count: usize,
    // Sizes
    pub bits: u32,
    pub mask_bits: u32,
    pub mask_count: usize,
    // Arrays
    pub table: FlagTables<T, TOTAL_COUNT, SINGLE_COUNT, GROUP_COUNT>,
}