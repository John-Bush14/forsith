use std::io::Read;
use const_for::const_for;
use derive_more::Not;
use crate::{DecodingError};
use crate::simd::SIMD_WIDTH;
use core::simd::prelude::*;

pub const POLY: u32 = 0xedb88320;
const CRC_TABLES: [[u32; 256]; 8] = const {
    let mut tables = [[0u32; 256]; 8];

    const_for!(n in 0..255+1 => {
        let mut c = n as u32;
        const_for!(_ in 0..8 => {
            if c & 1 == 1 {
                c = POLY ^ (c >> 1);
                continue;
            }
            c >>= 1;
        });
        tables[0][n as usize] = c
    });

    const_for!(n in 1..8 => {
        const_for!(k in 0..255+1 => {
            let crc = tables[n-1][k];
            tables[n][k] = tables[0][(crc & 0xff) as usize] ^ (crc >> 8);
        });
    });

    tables
};
const CRC_INIT: u32 = 0xFFFF_FFFF;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Not)]
pub struct CRC32(u32);
impl Default for CRC32 {
    fn default() -> Self {
        CRC32(CRC_INIT)
    }
}

impl CRC32 {
    pub fn update(&mut self, buf: &[u8]) {
        let (chunks, remainder) = buf.as_chunks::<8>();

        for chunk in chunks {
            let chunk: u64 = u64::from_le_bytes(*chunk);

            let x = self.0 as u64 ^ chunk;

            self.0 = CRC_TABLES[7][(x & 0xff) as usize]
                ^ CRC_TABLES[6][((x >> 8) & 0xff) as usize]
                ^ CRC_TABLES[5][((x >> 16) & 0xff) as usize]
                ^ CRC_TABLES[4][((x >> 24) & 0xff) as usize]
                ^ CRC_TABLES[3][((x >> 32) & 0xff) as usize]
                ^ CRC_TABLES[2][((x >> 40) & 0xff) as usize]
                ^ CRC_TABLES[1][((x >> 48) & 0xff) as usize]
                ^ CRC_TABLES[0][((x >> 56) & 0xff) as usize];
        }

        for b in remainder {
            self.0 = CRC_TABLES[0][((self.0 ^ *b as u32) & 0xff) as usize] ^ (self.0 >> 8);
        }
    }

    pub(crate) fn validate(&self, stored_crc: u32) -> Result<(), DecodingError> {
        let stored_crc = CRC32(stored_crc);

        let calculated_crc = !self.clone();

        if calculated_crc != stored_crc {
            return Err(DecodingError::CRCMismatch(calculated_crc, stored_crc));
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Adler32{
    a: u32,
    b: u32,
}
impl Default for Adler32 {
    fn default() -> Self {
        Adler32{ a: 1, b: 0}
    }
}
const ADLER_MOD: u32 = 65521;
const ADLER_CHUNK_SIZE: u16 = 5552 - (5552 % SIMD_WIDTH as u16);

impl Adler32 {
    pub fn update(&mut self, data: &[u8]) {
        let (chunks, remainder) = data.as_chunks::<{ADLER_CHUNK_SIZE as usize}>();

        for chunk in chunks {self.compute_chunk::<true>(chunk)}

        let unaligned_bytes = remainder.len() % SIMD_WIDTH;
        self.compute_chunk::<false>(&remainder[..remainder.len()-unaligned_bytes]);

        for b in remainder[remainder.len()-unaligned_bytes..].iter() {
            self.a += *b as u32;
            self.b += self.a;
        }

        self.a %= ADLER_MOD;
        self.b %= ADLER_MOD;
    }

    #[inline(always)]
    pub fn compute_chunk<const FULL_CHUNK: bool>(&mut self, chunk: &[u8]) {
        let (a, delta_b) = compute_alder32_chunk_simd(chunk, self.a);
        self.a = a;
        self.b += delta_b;

        if FULL_CHUNK {
            self.a %= ADLER_MOD;
            self.b %= ADLER_MOD;
        }
    }

    pub fn validate(&mut self, stored: u32) -> Result<(), DecodingError> {
        let computed = (self.b << 16) | self.a;

        if computed != stored {
            return Err(DecodingError::Adler32Mismatch(computed, stored));
        }

        Ok(())
    }
}

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

#[inline(always)]
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
