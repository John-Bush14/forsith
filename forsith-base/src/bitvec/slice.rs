use std::{fmt::Debug, ops::{Deref, Index}};

pub struct BitSlice {
    data: [()],
}

impl Debug for BitSlice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BitSlice")
            .field("bits", &self.bits())
            .field("bytes", &self.as_bools())
            .finish()
    }
}

impl PartialEq for BitSlice {
    fn eq(&self, other: &Self) -> bool {
        if self.bits() != other.bits() {return false}

        let full_bytes = self.bits() / 8;
        let remaining_bits_mask = (1 << (self.bits() % 8)) - 1;

        self.bytes()[..full_bytes] == other.bytes()[..full_bytes]
            && (self.bytes()[full_bytes] & remaining_bits_mask) == (other.bytes()[full_bytes] & remaining_bits_mask)
    }
}

impl Index<usize> for BitSlice {
    type Output = bool;

    fn index(&self, index: usize) -> &Self::Output {self.get(index).expect("index out of bounds")}
}

impl From<&BitSlice> for Vec<bool> {
    fn from(slice: &BitSlice) -> Self {
        slice.iter().collect()
    }
}

impl<'a> IntoIterator for &'a BitSlice {
    type Item = bool;
    type IntoIter = BitSliceIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        BitSliceIter { slice: self, index: 0 }
    }
}

impl<'a> IntoIterator for &'a mut BitSlice {
    type Item = MutBit<'a>;
    type IntoIter = MutBitSliceIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        MutBitSliceIter { slice: self, index: 0 }
    }
}

pub struct MutBit<'a> {
    slice: &'a mut BitSlice,
    index: usize,
}

impl MutBit<'_> {
    pub const fn set(&mut self, bit: bool) {
        self.slice.set(self.index, bit);
    }

    pub fn get(&self) -> bool {
        *self.slice.get(self.index).expect("MutBit index should always be valid")
    }
}

impl Deref for MutBit<'_> {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        self.slice.get(self.index).expect("MutBit index should always be valid")
    }
}

#[derive(Clone)]
pub struct BitSliceIter<'a> {
    slice: &'a BitSlice,
    index: usize,
}

impl Iterator for BitSliceIter<'_> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        let bit = *self.slice.get(self.index)?;
        self.index += 1;
        Some(bit)
    }
}

impl ExactSizeIterator for BitSliceIter<'_> {
    fn len(&self) -> usize {
        self.slice.bits() - self.index
    }
}

pub struct MutBitSliceIter<'a> {
    slice: &'a mut BitSlice,
    index: usize,
}

impl<'a> Iterator for MutBitSliceIter<'a> {
    type Item = MutBit<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.slice.bits() {return None}

        let bit = MutBit {
            slice: unsafe {self.slice.with_lifetime_mut()},
            index: self.index,
        };

        self.index += 1;

        Some(bit)
    }
}

impl ExactSizeIterator for MutBitSliceIter<'_> {
    fn len(&self) -> usize {
        self.slice.bits() - self.index
    }
}

impl BitSlice {
    #[must_use]
    pub fn iter(&self) -> BitSliceIter<'_> {self.into_iter()}

    #[must_use]
    pub fn iter_mut(&mut self) -> MutBitSliceIter<'_> {self.into_iter()}

    #[must_use]
    pub const fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr().cast()
    }
    #[must_use]
    pub const fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_mut_ptr().cast()
    }

    #[must_use]
    pub const fn from_bytes(bytes: &[u8], bits: usize) -> &Self {
        assert!(bits <= bytes.len() * 8, "new length exceeds capacity");
        unsafe {Self::from_raw_parts(bytes.as_ptr(), bits)}
    }

    #[must_use]
    pub fn from_bytes_mut(bytes: &mut [u8], bits: usize) -> &mut Self {
        assert!(bits <= bytes.len() * 8, "new length exceeds capacity");
        unsafe {Self::from_raw_parts_mut(bytes.as_mut_ptr(), bits)}
    }

    #[must_use]
    pub fn as_bools(&self) -> Vec<bool> {self.into()}

    const fn get_byte_bit_index(&self, index: usize) -> Option<(u8, usize)> {
        if index >= self.bits() {return None}
        let byte = unsafe {*self.as_ptr().add(index / 8)};

        Some((byte, index % 8))
    }

    const fn get_mut_byte_bit_index(&mut self, index: usize) -> Option<(&mut u8, usize)> {
        if index >= self.bits() {return None}
        let byte = unsafe {&mut *self.as_mut_ptr().add(index / 8)};

        Some((byte, index % 8))
    }

    #[must_use]
    pub const fn bytes(&self) -> &[u8] {
        unsafe {std::slice::from_raw_parts(self.as_ptr(), self.len_bytes())}
    }

    #[must_use]
    pub const fn bytes_mut(&mut self) -> &mut [u8] {
        unsafe {std::slice::from_raw_parts_mut(self.as_mut_ptr(), self.len_bytes())}
    }

    #[must_use]
    pub const fn bits(&self) -> usize {
        self.data.len()
    }

    #[must_use]
    pub const fn len_bytes(&self) -> usize {
        self.data.len().div_ceil(8)
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// # Safety
    /// The caller must ensure that the pointer is valid for reads of `bits` bits and that the
    /// lifetime of the returned reference does not outlive the lifetime of the data pointed to by
    /// `ptr`.
    #[must_use]
    pub const unsafe fn from_raw_parts<'a>(ptr: *const u8, bits: usize) -> &'a Self {
        unsafe {
            let wptr = std::ptr::slice_from_raw_parts(ptr, bits);

            &*(wptr as *const Self)
        }
    }

    /// # Safety
    /// The caller must ensure that the pointer is valid for read and writes of `bits` bits and that the
    /// lifetime of the returned reference does not outlive the lifetime of the data pointed to by
    /// `ptr`.
    #[must_use]
    pub unsafe fn from_raw_parts_mut<'a>(ptr: *mut u8, bits: usize) -> &'a mut Self {
        unsafe {
            let wptr = std::ptr::slice_from_raw_parts_mut(ptr, bits);

            &mut *(wptr as *mut Self)
        }
    }

    pub const fn set(&mut self, index: usize, bit: bool) {
        let (byte, bit_i) = self.get_mut_byte_bit_index(index).expect("index out of bounds");

        if bit {
            *byte |= 1 << bit_i;
        } else {
             *byte &= !(1 << bit_i);
        }
    }

    fn get(&self, index: usize) -> Option<&bool> {
        let (byte, bit_i) = self.get_byte_bit_index(index)?;

        Some(if byte & (1 << bit_i) != 0 {
            &true
        } else {
            &false
        })
    }

    /// # Safety
    /// The caller must ensure that the lifetime of the returned reference does not outlive the
    /// lifetime of the `BitSlice` reference.
    #[must_use]
    pub unsafe fn with_lifetime<'d>(&self) -> &'d Self {
        unsafe {std::mem::transmute(self)}
    }

    /// # Safety
    /// The caller must ensure that the lifetime of the returned reference does not outlive the
    /// lifetime of the `BitSlice` reference.
    #[must_use]
    pub unsafe fn with_lifetime_mut<'d>(&mut self) -> &'d mut Self {
        unsafe {std::mem::transmute(self)}
    }
}

#[cfg(test)]
mod bitslice_tests {
    use super::*;

    #[test]
    fn test_bitslice_get() {
        let bytes = [0b1010_1010, 0b1100_1100];
        let bitslice = BitSlice::from_bytes(&bytes, 16);

        for i in 0..16 {
            let expected = (bytes[i / 8] >> (i % 8)) & 1 == 1;
            assert_eq!(bitslice.get(i), Some(&expected));
        }

        assert_eq!(bitslice.get(16), None);
    }

    #[test]
    fn test_bitslice_set() {
        let mut bytes = [0b0000_0000, 0b0000_0000];
        let bitslice = BitSlice::from_bytes_mut(&mut bytes, 16);

        for i in 0..16 {
            bitslice.set(i, i % 2 == 0);
        }

        assert_eq!(bytes, [0b0101_0101, 0b0101_0101]);
    }

    #[test]
    fn test_bitslice_iter() {
        let bytes = [0b1010_1010, 0b1100_1100];
        let bitslice = BitSlice::from_bytes(&bytes, 16);

        let expected: Vec<bool> = (0..16).map(|i| (bytes[i / 8] >> (i % 8)) & 1 == 1).collect();
        let actual: Vec<bool> = bitslice.iter().collect();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_bitslice_iter_mut() {
        let mut bytes = [0b0000_0000, 0b0000_0000];
        let bitslice = BitSlice::from_bytes_mut(&mut bytes, 16);

        for (i, mut mut_bit) in bitslice.iter_mut().enumerate() {
            mut_bit.set(i % 2 == 0);
        }

        assert_eq!(bytes, [0b0101_0101, 0b0101_0101]);
    }

    #[test]
    fn test_bitslice_partial_eq() {
        let bytes1 = [0b1010_1010, 0b1100_1100];
        let bitslice1 = BitSlice::from_bytes(&bytes1, 14);

        let bytes2 = [0b1010_1010, 0b0000_1100];
        let bitslice2 = BitSlice::from_bytes(&bytes2, 14);

        assert_eq!(bitslice1, bitslice2);
    }

    #[test]
    fn test_bitslice_partial_eq_different_lengths() {
        let bytes1 = [0b1010_1010, 0b1100_1100];
        let bitslice1 = BitSlice::from_bytes(&bytes1, 14);
        let bitslice2 = BitSlice::from_bytes(&bytes1, 15);
        assert_ne!(bitslice1, bitslice2);
    }

    #[test]
    fn test_bitslice_partial_eq_different_values() {
        let bytes1 = [0b1010_1010, 0b1100_1000];
        let bitslice1 = BitSlice::from_bytes(&bytes1, 14);

        let bytes2 = [0b1010_1010, 0b1100_1111];
        let bitslice2 = BitSlice::from_bytes(&bytes2, 14);

        assert_ne!(bitslice1, bitslice2);
    }
}
