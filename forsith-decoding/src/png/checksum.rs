use std::{io::BufRead, ops::Not};
use const_for::const_for;
use crate::{DecodingError, png::{reader::{BitReader, Reader}, simd::{SIMD_WIDTH, checksum::compute_alder32_chunk_simd}}, read_exact_array};

pub const POLY: u32 = 0xedb88320;
const CRC_TABLE: [u32; 256] = const {
    let mut table = [0u32; 256];

    const_for!(n in 0..255+1 => {
        let mut c = n as u32;
        const_for!(_ in 0..8 => {
            if c & 1 == 1 {
                c = POLY ^ (c >> 1);
                continue;
            }
            c >>= 1;
        });
        table[n as usize] = c
    });

    table
};
const CRC_INIT: u32 = 0xFFFF_FFFF;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct CRC32(u32);
impl Default for CRC32 {
    fn default() -> Self {
        CRC32(CRC_INIT)
    }
}

impl Not for CRC32 {
    type Output = Self;

    fn not(self) -> Self::Output {
        CRC32(!self.0)
    }
}

impl CRC32 {
    pub fn update(&mut self, buf: &[u8]) {
        for b in buf {
            self.0 = CRC_TABLE[((self.0 ^ *b as u32) & 0xff) as usize] ^ (self.0 >> 8);
        }
    }

    pub fn finalize(&mut self) -> u32 {
        !self.0
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

impl<R: BufRead> Reader<R> {
    pub fn validate_crc(&mut self) -> Result<(), DecodingError> {
        let stored_crc = CRC32(u32::from_be_bytes(read_exact_array::<4,_>(self.normal_reader())?));

        self.crc = !self.crc;

        if self.crc != stored_crc {
            return Err(DecodingError::CRCMismatch(self.crc, stored_crc));
        }

        Ok(())
    }

    pub fn update_crc(&mut self, buf: &[u8]) {self.crc.update(buf)}

    pub fn reset_crc(&mut self) {self.crc = CRC32::default()}

    pub fn update_adler32(&mut self, data: &[u8]) {
        let (chunks, remainder) = data.as_chunks::<{ADLER_CHUNK_SIZE as usize}>();

        for chunk in chunks {self.compute_alder32_chunk::<true>(chunk)}

        let unaligned_bytes = remainder.len() % SIMD_WIDTH;
        self.compute_alder32_chunk::<false>(&remainder[..remainder.len()-unaligned_bytes]);

        for b in remainder[remainder.len()-unaligned_bytes..].iter() {
            self.adler.a += *b as u32;
            self.adler.b += self.adler.a;
        }

        self.adler.a %= ADLER_MOD;
        self.adler.b %= ADLER_MOD;
    }

    pub fn compute_alder32_chunk<const FULL_CHUNK: bool>(&mut self, chunk: &[u8]) {
        let (a, delta_b) = compute_alder32_chunk_simd(chunk, self.adler.a);
        self.adler.a = a;
        self.adler.b += delta_b;

        if FULL_CHUNK {
            self.adler.a %= ADLER_MOD;
            self.adler.b %= ADLER_MOD;
        }
    }

    pub fn validate_adler32(&mut self) -> Result<(), DecodingError> {
        if self.remaining_bytes > 4 {
            return Err(DecodingError::IncorrectClose(self.cur_type(), self.remaining_bytes));
        }

        self.consume_bits(self.bit_buf.bits_remaining() % 8);
        let stored_adler = ((self.read_bits(8)? as u32) << 24)
            | ((self.read_bits(8)? as u32) << 16)
            | ((self.read_bits(8)? as u32) << 8)
            | (self.read_bits(8)? as u32);

        let computed_adler = ((self.adler.b % ADLER_MOD) << 16) | (self.adler.a % ADLER_MOD);

        if computed_adler != stored_adler {
            return Err(DecodingError::Adler32Mismatch(computed_adler, stored_adler));
        }

        Ok(())
    }
}
