use crate::png::simd::SIMD_WIDTH;
use core::simd::prelude::*;


const POSITIONS: Simd<AdlerLaneSize, SIMD_WIDTH> = {
    let mut arr = [0; SIMD_WIDTH];
    let mut i = 0;

    while i < SIMD_WIDTH {
        arr[i] = SIMD_WIDTH as AdlerLaneSize - i as AdlerLaneSize;
        i += 1;
    }

    Simd::from_array(arr)
};



type AdlerLaneSize = u16;

pub fn compute_alder32_chunk_simd(chunk: &[u8], mut a: u32) -> (u32, u32) {
    let mut b = 0u32;

    for chunk in chunk.as_chunks::<SIMD_WIDTH>().0 {
        let chunkv = Simd::<u8, SIMD_WIDTH>::from_slice(chunk).cast::<AdlerLaneSize>();

        let sum = chunkv.reduce_sum();

        let weightedv = chunkv * POSITIONS;
        let weighted_sum = weightedv.reduce_sum();

        let delta_b = weighted_sum as u32 + a * SIMD_WIDTH as u32;

        a += sum as u32;
        b += delta_b;
    }

    (a, b)
}
