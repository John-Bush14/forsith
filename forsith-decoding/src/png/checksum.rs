use std::{io::BufRead, ops::Not};
use const_for::const_for;
use crate::{DecodingError, png::{reader::{BitReader, PngReader}, simd::{SIMD_WIDTH, checksum::compute_alder32_chunk_simd}}};

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

impl<R: BufRead> PngReader<R> {
    pub fn validate_crc(&mut self, stored_crc: u32) -> Result<(), DecodingError> {
        let stored_crc = CRC32(stored_crc);

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
        if self.bit_buf.bits_remaining() < 32 {
            self.fill_bitbuf()?;
        }

        self.consume_bits(self.bit_buf.bits_remaining() % 8);
        let stored_adler = (self.bit_buf.peek(32) as u32).to_be();
        self.bit_buf.consume(32);


        let stolen_bytes = self.bit_buf.bits_remaining() as usize / 8;

        self.buffer.unconsume(stolen_bytes);
        self.buffer.mut_slice(stolen_bytes).copy_from_slice(&self.bit_buf.peek(stolen_bytes as u8*8).to_be_bytes()[..stolen_bytes]);
        self.bit_buf.consume(stolen_bytes as u8);

        let computed_adler = ((self.adler.b % ADLER_MOD) << 16) | (self.adler.a % ADLER_MOD);

        if computed_adler != stored_adler {
            return Err(DecodingError::Adler32Mismatch(computed_adler, stored_adler));
        }

        Ok(())
    }
}
