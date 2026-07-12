use crate::{DecodingError, png::filtering::Filterer};
use core::simd::prelude::*;

pub use super::SIMD_WIDTH;

use super::open_simd;

pub const fn should_use_simd<const STRIDE: usize, const FILTER: u8>() -> bool {
    if FILTER == 2 {return true}

    if FILTER == 1 && matches!(STRIDE, 4 | 8) {return true}

    return false
}

impl Filterer {
    pub fn filter_simd<const FILTER: u8>(&self, scanline: &[u8], i: usize) -> Result<Simd<u8, SIMD_WIDTH>, DecodingError> {
        let raw_bytes = open_simd(scanline, i);

        let filtered_bytes = match FILTER {
            1 => match self.stride  {
                4 => sub_filter::<4>(raw_bytes, scanline[i - 4..i].try_into().unwrap()),
                8 => sub_filter::<8>(raw_bytes, scanline[i - 8..i].try_into().unwrap()),
                _ => unreachable!()
            },
            2 => raw_bytes + self.upper_pixels(i),
            3 => todo!(),
            4 => todo!(),
            _ => return Err(DecodingError::InvalidFilter(FILTER)),
        };

        Ok(filtered_bytes)
    }

    /// only first {self.stride} pixels correct, others will be garbage
    fn upper_pixels(&self, i: usize) -> Simd<u8, SIMD_WIDTH> {open_simd(self.prev_buffer(), i)}
    fn _left_upper_pixels(&self, i: usize) -> Simd<u8, SIMD_WIDTH> {open_simd(self.prev_buffer(), i - self.stride)}
}

fn sub_filter<const STRIDE: usize>(mut raw_bytes: Simd<u8, SIMD_WIDTH>, left_pixel: [u8; STRIDE]) -> Simd<u8, SIMD_WIDTH> {
    let mut shifted_bytes = raw_bytes;

    for _ in (0..SIMD_WIDTH).step_by(STRIDE) {
        shifted_bytes = shifted_bytes.shift_elements_right::<STRIDE>(0);
        raw_bytes += shifted_bytes
    }

    let anchor = array_repeating_to_simd(left_pixel);

    raw_bytes + anchor
}

fn array_repeating_to_simd<const LENGTH: usize>(arr: [u8; LENGTH]) -> Simd<u8, SIMD_WIDTH> {
    Simd::from_array(arr).resize::<{SIMD_WIDTH}>(0).swizzle_dyn(Simd::from_array(repeating_swizzle_index::<{LENGTH}>()))
}

const fn repeating_swizzle_index<const MAX_INDEX: usize>() -> [u8; SIMD_WIDTH] {
    let mut arr = [0; SIMD_WIDTH];
    let mut i = 0;

    while i < SIMD_WIDTH {
        arr[i] = (i % MAX_INDEX) as u8;
        i += 1;
    }

    arr
}
