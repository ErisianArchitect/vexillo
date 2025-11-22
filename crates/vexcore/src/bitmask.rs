
pub type BitmaskType = u64;

const BITMASK_BITS: u32 = BitmaskType::BITS;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Bitmask {
    pub masks: Box<[BitmaskType]>,
    pub bit_count: u32,
}

    
#[inline]
#[must_use]
fn mask_indices(index: u32) -> (usize, u32) {
    let mask_index = index / BITMASK_BITS;
    let bit_index = index % BITMASK_BITS;
    (
        mask_index as usize,
        bit_index,
    )
}

impl Bitmask {
    #[inline]
    #[must_use]
    pub fn new(bit_count: u32) -> Self {
        let mask_count = (bit_count + (BITMASK_BITS - 1)) / BITMASK_BITS;
        Self {
            masks: Box::from_iter((0..mask_count).map(|_| 0)),
            bit_count,
        }
    }
    
    #[inline]
    pub fn new_flag(bit_count: u32, bit_index: u32) -> Self {
        Self::new(bit_count).with_bit(bit_index)
    }
    
    #[inline]
    #[must_use]
    pub fn with_bit(mut self, index: u32) -> Self {
        let (mask, bit) = mask_indices(index);
        self.masks[mask] |= 1 << bit;
        self
    }
    
    #[inline]
    pub fn add(&mut self, other: &Self) -> &mut Self {
        ::core::iter::zip(
            self.masks.iter_mut(),
            other.masks.iter().copied(),
        ).for_each(|(lhs, rhs)| {
            *lhs |= rhs;
        });
        self
    }
    
    #[inline]
    pub fn remove(&mut self, other: &Self) -> &mut Self {
        ::core::iter::zip(
            self.masks.iter_mut(),
            other.masks.iter().copied(),
        ).for_each(|(lhs, rhs)| {
            *lhs &= !rhs;
        });
        self
    }
    
    #[inline]
    pub fn get_bit(&self, index: u32) -> bool {
        let (mask, bit) = mask_indices(index);
        let bitmask = 1 << bit;
        (self.masks[mask] & bitmask) == bitmask
    }
    
    #[inline]
    pub fn set_bit(&mut self, index: u32, on: bool) {
        let (mask, bit) = mask_indices(index);
        let bitmask = 1 << bit;
        if on {
            self.masks[mask] |= bitmask;
        } else {
            self.masks[mask] &= !bitmask;
        }
    }
    
    #[inline]
    #[must_use]
    pub fn count_ones(&self) -> u32 {
        self.masks
            .iter()
            .copied()
            .fold(0, |count, mask| {
                count + mask.count_ones()
            })
    }
}