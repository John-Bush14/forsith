use crate::{DecodingError, png::postprocessing::PostProcessor};
use core::simd::prelude::*;

pub use super::SIMD_WIDTH;

use super::open_simd;

pub const fn should_use_simd<const STRIDE: usize, const FILTER: u8>() -> bool {
    if FILTER == 1 && STRIDE >= 3 {return true}

    if FILTER == 2 {return true}

    if FILTER == 3 && STRIDE >= 6 {return true}

    false
}

impl PostProcessor {
    #[inline(always)]
    pub fn filter_simd<const FILTER: u8, const STRIDE: usize>(&self, scanline: &[u8], i: usize) -> Result<Simd<u8, SIMD_WIDTH>, DecodingError> {
        let raw_bytes = open_simd(scanline, i);

        let filtered_bytes = match FILTER {
            1 => sub_filter::<STRIDE>(raw_bytes, self.left_pixel::<STRIDE>(i)),
            2 => raw_bytes + self.upper_pixels(i),
            3 => average_filter::<STRIDE>(raw_bytes, self.left_pixels::<STRIDE>(i), self.upper_pixels(i)),
            4 => todo!(),
            _ => return Err(DecodingError::InvalidFilter(FILTER)),
        };

        Ok(filtered_bytes)
    }

    /// only first {self.stride} pixels correct, others will be garbage
    fn left_pixel<const STRIDE: usize>(&self, i: usize) -> [u8; STRIDE] {
        self.cur_buffer().slice(i - STRIDE..i).try_into().unwrap()
    }
    fn left_pixels<const STRIDE: usize>(&self, i: usize) -> Simd<u8, SIMD_WIDTH> {
        let mut left_pixels = Simd::splat(0);
        left_pixels.as_mut_array()[..STRIDE].copy_from_slice(&self.left_pixel::<STRIDE>(i));
        left_pixels
    }
    fn upper_pixels(&self, i: usize) -> Simd<u8, SIMD_WIDTH> {open_simd(self.prev_buffer().as_slice(), i)}
    fn _left_upper_pixels(&self, i: usize) -> Simd<u8, SIMD_WIDTH> {open_simd(self.prev_buffer().as_slice(), i - self.stride)}
}

fn average_filter<const STRIDE: usize>(mut raw_bytes: Simd<u8, SIMD_WIDTH>, left_pixels: Simd<u8, SIMD_WIDTH>, mut upper_pixels: Simd<u8, SIMD_WIDTH>) -> Simd<u8, SIMD_WIDTH> {
    raw_bytes += simd_average(left_pixels, upper_pixels);

    let mut shifted_bytes = raw_bytes;

    for _ in (STRIDE..SIMD_WIDTH).step_by(STRIDE) {
        shifted_bytes = shifted_bytes.shift_elements_right::<STRIDE>(0);
        upper_pixels = upper_pixels.shift_elements_right::<STRIDE>(0);

        raw_bytes += simd_average(shifted_bytes, upper_pixels);
    }

    raw_bytes
}
fn simd_average(a: Simd<u8, SIMD_WIDTH>, b: Simd<u8, SIMD_WIDTH>) -> Simd<u8, SIMD_WIDTH> {
    (a & b) + ((a ^ b) >> Simd::splat(1))
}

fn sub_filter<const STRIDE: usize>(mut raw_bytes: Simd<u8, SIMD_WIDTH>, left_pixel: [u8; STRIDE]) -> Simd<u8, SIMD_WIDTH> {
    let mut shifted_bytes = raw_bytes;

    for _ in (STRIDE..SIMD_WIDTH).step_by(STRIDE) {
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
