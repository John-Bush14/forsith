use std::ops::{Deref, DerefMut};

use crate::bitvec::BitSlice;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BitArray<const BYTES: usize>([u8; BYTES]);

impl<const BYTES: usize> Deref for BitArray<BYTES> {
    type Target = BitSlice;

    fn deref(&self) -> &Self::Target {
        BitSlice::from_raw_parts(self.0.as_ptr(), self.bits())
    }
}

impl<const BYTES: usize> DerefMut for BitArray<BYTES> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        BitSlice::from_raw_parts_mut(self.0.as_mut_ptr(), self.bits())
    }
}

impl<const BYTES: usize> BitArray<BYTES> {
    #[must_use]
    pub const fn filled(value: bool) -> Self {
        let byte = if value { 0xFF } else { 0x00 };
        Self([byte; BYTES])
    }

    #[must_use]
    pub const fn bits(&self) -> usize {
        BYTES * 8
    }

    #[must_use]
    pub const fn len_bytes(&self) -> usize {
        BYTES
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        BYTES == 0
    }
}
