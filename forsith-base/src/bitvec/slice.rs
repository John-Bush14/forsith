use std::ops::{Deref, Index};

pub struct BitSlice {
    data: [()],
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
    pub fn as_bools(&self) -> Vec<bool> {self.into()}

    const fn get_byte_bit_index(&self, index: usize) -> Option<(u8, usize)> {
        let index = index / 8;
        if index >= self.bits() {return None}
        let byte = unsafe {*self.as_ptr().add(index)};

        Some((byte, index % 8))
    }

    const fn get_mut_byte_bit_index(&mut self, index: usize) -> Option<(&mut u8, usize)> {
        if index >= self.bits() {return None}
        let index = index / 8;
        let byte = unsafe {&mut *self.as_mut_ptr().add(index)};

        Some((byte, index % 8))
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

    #[must_use]
    pub const fn from_raw_parts<'a>(ptr: *const u8, bits: usize) -> &'a Self {
        unsafe {
            let wptr = std::ptr::slice_from_raw_parts(ptr, bits);

            &*(wptr as *const Self)
        }
    }

    #[must_use]
    pub fn from_raw_parts_mut<'a>(ptr: *mut u8, bits: usize) -> &'a mut Self {
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
