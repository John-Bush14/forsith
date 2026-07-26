use crate::{Channel, png::postprocessing::PostProcessor};
use core::simd::prelude::*;

pub use super::SIMD_WIDTH;

use super::open_simd;

pub const fn should_use_simd<const FILTER: u8>(stride: usize) -> bool {
    if FILTER == 1 && stride >= 3 {return true}

    if FILTER == 2 {return true}

    // if FILTER == 3 && stride >= 6 {return true}

    false
}

impl<C: Channel, const F: u8> PostProcessor<C, F> {
    #[inline(always)]
    pub fn filter_simd<const FILTER: u8, const STRIDE: usize>(&self, cur: *mut u8, up: *mut u8) {
        let raw_bytes = open_simd(cur);

        let result = match FILTER {
            1 => sub_filter::<STRIDE>(raw_bytes, self.left_pixel::<STRIDE>(cur)),
            2 => raw_bytes + self.upper_pixels(up),
            3 => average_filter::<STRIDE>(raw_bytes, self.left_pixels::<STRIDE>(cur), self.upper_pixels(up)),
            4 => todo!(),
            _ => unreachable!(),
        };

        unsafe {
            cur.copy_from_nonoverlapping(result.as_array().as_ptr(), SIMD_WIDTH);
        }
    }

    fn left_pixel<const STRIDE: usize>(&self, cur: *mut u8) -> &[u8; STRIDE] {
        unsafe {
            std::slice::from_raw_parts(cur.sub(STRIDE), STRIDE).try_into().unwrap_unchecked()
        }
    }
    /// only first {self.stride} pixels correct, others 0
    fn left_pixels<const STRIDE: usize>(&self, cur: *mut u8) -> Simd<u8, SIMD_WIDTH> {
        let mut left_pixels = Simd::splat(0);
        left_pixels.as_mut_array()[..STRIDE].copy_from_slice(self.left_pixel::<STRIDE>(cur));
        left_pixels
    }
    fn upper_pixels(&self, up: *mut u8) -> Simd<u8, SIMD_WIDTH> {
        open_simd(up)
    }
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

#[inline]
fn sub_filter<const STRIDE: usize>(mut raw_bytes: Simd<u8, SIMD_WIDTH>, left_pixel: &[u8; STRIDE]) -> Simd<u8, SIMD_WIDTH> {
    let mut shifted_bytes = raw_bytes;

    for _ in (STRIDE..SIMD_WIDTH).step_by(STRIDE) {
        shifted_bytes = shifted_bytes.shift_elements_right::<STRIDE>(0);
        raw_bytes += shifted_bytes
    }

    let anchor = array_repeating_to_simd(left_pixel);

    raw_bytes + anchor
}

#[inline]
fn array_repeating_to_simd<const LENGTH: usize>(arr: &[u8; LENGTH]) -> Simd<u8, SIMD_WIDTH> {
    Simd::<u8, LENGTH>::from_slice(arr).resize::<{SIMD_WIDTH}>(0).swizzle_dyn(Simd::from_array(repeating_swizzle_index::<{LENGTH}>()))
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
