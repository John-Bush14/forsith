use core::simd::prelude::*;

// make AdlerLaneSize u32 if increasing SIMD_WIDTH above 16, otherwise u16 is enough
pub const SIMD_WIDTH: usize = 16;

#[inline(always)]
pub const fn open_simd(slice: &[u8]) -> Simd::<u8, {SIMD_WIDTH}> {
    Simd::from_slice(slice)
}

