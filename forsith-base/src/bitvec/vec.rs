use crate::buffer;
use std::{mem::MaybeUninit, ops::{Deref, DerefMut}};

use crate::{bitvec::{ BitSlice}, buffer::Buffer};

#[derive(Debug, Clone, Default)]
pub struct BitVec {
    data: Buffer<MaybeUninit<u8>>,
    bits: usize,
}

impl Deref for BitVec {
    type Target = BitSlice;

    fn deref(&self) -> &Self::Target {
        BitSlice::from_raw_parts(self.data.as_ptr().cast::<u8>(), self.bits)
    }
}

impl DerefMut for BitVec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        BitSlice::from_raw_parts_mut(self.data.as_mut_ptr().cast::<u8>(), self.bits)
    }
}

impl BitVec {
    #[must_use]
    pub fn new() -> Self {Self::default()}

    /// Creates a new `BitVec` with the next multiple of 8 of 'bits' as it's capacity.
    #[must_use]
    pub fn with_capacity(bits: usize) -> Self {
        let bytes = bits.div_ceil(8);

        let data = buffer![MaybeUninit::uninit(); bytes];

        Self { data, bits }
    }

    #[must_use]
    pub fn capacity(&self) -> usize {
        self.data.len() * 8
    }

    #[must_use]
    pub const fn bits(&self) -> usize {
        self.bits
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.bits
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.bits == 0
    }

    /// # Safety
    /// The caller must ensure that the new length does not exceed the capacity of the `BitVec`,
    pub unsafe fn set_len(&mut self, bits: usize) {
        #[cfg(debug_assertions)]
        assert!(bits <= self.capacity(), "new length exceeds capacity");
        self.bits = bits;
    }

    pub fn grow_if_needed(&mut self, new_bits: usize) {
        let new_bytes = new_bits.div_ceil(8);
        let old_bytes = self.data.len();

        if new_bytes > old_bytes {
            self.data.resize(new_bytes.next_power_of_two(), MaybeUninit::uninit());
        }
    }

    pub fn push(&mut self, bit: bool) {
        let new_bits = self.bits + 1;
        self.grow_if_needed(new_bits);

        self.bits = new_bits;

        self.set(new_bits - 1, bit);
    }
}
