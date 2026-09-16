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

impl From<&BitSlice> for BitVec {
    fn from(slice: &BitSlice) -> Self {
        let data = unsafe {Buffer::copy_from_ptr(slice.as_ptr().cast(), slice.len_bytes())};

        Self { data, bits: slice.bits() }
    }
}

impl From<&[bool]> for BitVec {
    fn from(slice: &[bool]) -> Self {Self::from_bools(slice)}
}

impl From<Buffer<u8>> for BitVec {
    fn from(buffer: Buffer<u8>) -> Self {
        let bits = buffer.len() * 8;
        let data = unsafe {std::mem::transmute::<Buffer<u8>, Buffer<MaybeUninit<u8>>>(buffer)};

        Self { data, bits }
    }
}

impl From<BitVec> for Buffer<MaybeUninit<u8>> {
    fn from(bitvec: BitVec) -> Self {bitvec.into_buffer()}
}

impl BitVec {
    #[must_use]
    pub fn new() -> Self {Self::default()}

    #[must_use]
    pub const fn buffer(&self) -> &Buffer<MaybeUninit<u8>> {&self.data}
    #[must_use]
    pub const fn buffer_mut(&mut self) -> &mut Buffer<MaybeUninit<u8>> {&mut self.data}
    #[must_use]
    pub fn into_buffer(self) -> Buffer<MaybeUninit<u8>> {self.data}
    #[must_use]
    pub fn from_buffer(buffer: Buffer<MaybeUninit<u8>>, bits: usize) -> Self {
        assert!(bits <= buffer.len() * 8, "new length exceeds capacity");
        Self { data: buffer, bits }
    }

    #[must_use]
    pub fn from_bools(bits: &[bool]) -> Self {
        let mut bitvec = Self::with_capacity(bits.len());

        bitvec.iter_mut().zip(bits.iter()).for_each(|(mut b, &bit)| b.set(bit));

        bitvec
    }

    /// Creates a new `BitVec` with the next multiple of 8 of 'bits' as it's capacity.
    #[must_use]
    pub fn with_capacity(bits: usize) -> Self {
        let bytes = bits.div_ceil(8);

        let data = buffer![MaybeUninit::uninit(); bytes];

        Self { data, bits }
    }

    #[must_use]
    pub fn filled(value: bool, len: usize) -> Self {
        let byte = if value { 0xFF } else { 0x00 };
        let data = buffer![MaybeUninit::new(byte); len];

        Self { data, bits: len }
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

#[cfg(test)]
mod bitvec_tests {
    use super::*;

    #[test]
    fn test_bitvec_push() {
        let mut bitvec = BitVec::new();

        for i in 0..10 {
            bitvec.push(i % 2 == 0);
        }

        assert_eq!(bitvec.len(), 10);
        assert_eq!(bitvec.capacity(), 16); // Next power of two of 10 bits is 16 bits (2 bytes)

        bitvec.iter().eq((0..10).map(|i| i % 2 == 0));
    }
}
