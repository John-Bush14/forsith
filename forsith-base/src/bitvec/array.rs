use std::ops::{Deref, DerefMut};

use crate::bitvec::BitSlice;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BitArray<const BYTES: usize>([u8; BYTES]);

impl<const BYTES: usize> Default for BitArray<BYTES> {
    fn default() -> Self {
        Self([0; BYTES])
    }
}

impl<const BYTES: usize> From<[u8; BYTES]> for BitArray<BYTES> {
    fn from(arr: [u8; BYTES]) -> Self {Self(arr)}
}
impl<const BYTES: usize> From<BitArray<BYTES>> for [u8; BYTES] {
    fn from(arr: BitArray<BYTES>) -> Self {arr.into_bytes()}
}

impl<const BYTES: usize> Deref for BitArray<BYTES> {
    type Target = BitSlice;

    fn deref(&self) -> &Self::Target {
        unsafe {BitSlice::from_raw_parts(self.0.as_ptr(), BYTES)}
    }
}

impl<const BYTES: usize> DerefMut for BitArray<BYTES> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {BitSlice::from_raw_parts_mut(self.0.as_mut_ptr(), BYTES)}
    }
}

impl<const BYTES: usize> BitArray<BYTES> {
    #[must_use]
    pub const fn filled(value: bool) -> Self {
        let byte = if value { 0xFF } else { 0x00 };
        Self([byte; BYTES])
    }

    #[must_use]
    pub const fn into_bytes(self) -> [u8; BYTES] {self.0}

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        BYTES == 0
    }
}
