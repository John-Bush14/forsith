use core::ops::{Deref, DerefMut};

use crate::bitvec::BitSlice;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BitArray<const BYTES: usize>([u8; BYTES]);

impl<const BYTES: usize> Default for BitArray<BYTES> {
    fn default() -> Self {
        Self([0; BYTES])
    }
}

impl<const BYTES: usize> From<[u8; BYTES]> for BitArray<BYTES> {
    fn from(arr: [u8; BYTES]) -> Self {
        Self(arr)
    }
}
impl<const BYTES: usize> From<BitArray<BYTES>> for [u8; BYTES] {
    fn from(arr: BitArray<BYTES>) -> Self {
        arr.into_bytes()
    }
}

impl<const BYTES: usize> Deref for BitArray<BYTES> {
    type Target = BitSlice;

    fn deref(&self) -> &Self::Target {
        unsafe { BitSlice::from_raw_parts(self.0.as_ptr(), BYTES * 8) }
    }
}

impl<const BYTES: usize> DerefMut for BitArray<BYTES> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { BitSlice::from_raw_parts_mut(self.0.as_mut_ptr(), BYTES * 8) }
    }
}

impl<const BYTES: usize> BitArray<BYTES> {
    #[must_use]
    pub const fn filled(value: bool) -> Self {
        let byte = if value { 0xFF } else { 0x00 };
        Self([byte; BYTES])
    }

    #[must_use]
    pub const fn into_bytes(self) -> [u8; BYTES] {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitarray_default_and_filled() {
        let default = BitArray::<4>::default();
        assert_eq!(default.into_bytes(), [0, 0, 0, 0]);

        let filled = BitArray::<4>::filled(true);
        assert_eq!(filled.into_bytes(), [0xFF, 0xFF, 0xFF, 0xFF]);
    }

    #[test]
    fn bitarray_uses_bit_slice_accessors() {
        let mut array = BitArray::<2>::from([0b1010_1010, 0b1100_1100]);

        assert!(!array[0]);
        assert!(array[1]);
        assert!(array[7]);

        array.set(0, true);
        array.set(15, false);

        assert!(array[0]);
        assert!(!array[15]);
        assert_eq!(array.into_bytes(), [0b1010_1011, 0b0100_1100]);
    }
}
