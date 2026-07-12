use core::simd::prelude::*;

pub mod filtering;

pub mod checksum;

// make AdlerLaneSize u32 if increasing SIMD_WIDTH above 16, otherwise u16 is enough
pub const SIMD_WIDTH: usize = 16;

fn open_simd(buf: &[u8], len: usize) -> Simd::<u8, {SIMD_WIDTH}> {
    unsafe {
      Simd::<u8, SIMD_WIDTH>::from_slice(buf.get_unchecked(len..len + SIMD_WIDTH))
    }
}

