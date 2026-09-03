use std::{io::{Read, Seek}, ops::{Deref, DerefMut}};

use crate::{buffers::CursorVec, int::Int};

pub trait BitRead {
    fn fill_bitbuf(&mut self);
    fn peek_bits(&mut self, n: u8) -> u64;
    fn peek_bits_nobranch(&mut self, n: u8) -> u64;
    fn consume_bits(&mut self, n: u8);
    fn remaining_bits(&self) -> u8;
    fn read_bits(&mut self, n: u8) -> u64 {
        let bits = self.peek_bits(n);
        self.consume_bits(n);
        bits
    }
    fn read_bits_nobranch(&mut self, n: u8) -> u64 {
        let bits = self.peek_bits_nobranch(n);
        self.consume_bits(n);
        bits
    }
    fn iterate_bits<const BITS: u8>(&mut self) -> BitIterator<'_, Self, BITS> where Self: Sized {
        BitIterator {
            reader: self,
        }
    }
}

pub struct BitIterator<'a, R: BitRead, const BITS: u8> {
    reader: &'a mut R,
}
impl<R: BitRead, const BITS: u8> Iterator for BitIterator<'_, R, BITS> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.reader.read_bits(BITS))
    }
}

#[derive(Debug, Default)]
pub struct BitBuffer {
    buf: u64,
    bits_remaining: u8
}
impl BitBuffer {
    #[inline(always)]
    pub const fn bits_remaining(&self) -> u8 {self.bits_remaining}

    #[inline(always)]
    pub const fn peek(&self, n: u8) -> u64 {
        self.buf & ((1 << n as usize) - 1)
    }

    #[inline(always)]
    pub const fn consume(&mut self, n: u8) {
        self.buf >>= n as usize;
        self.bits_remaining -= n;
    }

    #[inline(always)]
    pub fn push<T: Int>(&mut self, value: T) {
        assert!(T::MIN == 0, "BitBuffer.push should only be called with unsigned ints");

        let value: u64 = value.try_into().unwrap_or_else(|_| unreachable!());

        self.buf |= value << self.bits_remaining as usize;
        self.bits_remaining += T::BIT_DEPTH;
    }
}

#[derive(Debug, Default)]
pub struct BitReader<T: Read + Default + Seek> {
    buffer: T,
    bit_buf: BitBuffer,
}

impl<T: Read + Default + Seek> Deref for BitReader<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}
impl<T: Read + Default + Seek> DerefMut for BitReader<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

impl<T: Read + Default + Seek> BitReader<T> {
    pub fn new(buffer: T) -> Self {
        Self {
            buffer,
            bit_buf: BitBuffer::default(),
        }
    }

    pub fn align(&mut self) -> Result<(), std::io::Error> {
        let alignment = 4 - (self.buffer.stream_position().unwrap() as usize % align_of::<u32>());

        let mut buf = vec![0u8; alignment];
        self.buffer.read_exact(&mut buf)?;

        for b in buf {self.bit_buf.push(b)}; Ok(())
    }

    pub fn unconsume_bitbuf(&mut self) {
        let bitbuf_bytes = self.bit_buf.bits_remaining().div_euclid(8);

        self.buffer.seek_relative(-(bitbuf_bytes as i64)).unwrap();
        self.bit_buf.consume(self.bit_buf.bits_remaining());
    }
}

impl BitReader<CursorVec<u8>> {
    #[inline(always)]
    pub fn fill_bitbuf(&mut self) {
        // performance reasons
        let refil = u32::from_le_bytes(self.buffer.take_mut_slice(4).try_into().unwrap());

        self.bit_buf.push(refil);
    }
}

impl<T: Read + Default + Seek> BitRead for BitReader<T> {
    #[inline(always)]
    fn peek_bits(&mut self, n: u8) -> u64 {
        if self.bit_buf.bits_remaining() <= 32 {
            self.fill_bitbuf();
        }

        self.bit_buf.peek(n)
    }

    #[inline(always)]
    fn fill_bitbuf(&mut self) {
        let mut buf = [0u8; 4];
        self.buffer.read_exact(&mut buf).unwrap();
        let refill = u32::from_le_bytes(buf);
        self.bit_buf.push(refill);
    }

    #[inline(always)]
    fn consume_bits(&mut self, n: u8) {
        self.bit_buf.consume(n);
    }

    #[inline]
    fn remaining_bits(&self) -> u8 {
        self.bit_buf.bits_remaining
    }

    #[inline(always)]
    fn peek_bits_nobranch(&mut self, n: u8) -> u64 {self.bit_buf.peek(n)}
}

pub fn unpack<const UPSAMPLE: bool>(slice: &[u8], bits: u8, padding: u8, callback: impl FnMut(&[u8])) {
    (match bits {
       1 => unpack_constant::<1, UPSAMPLE>,
       2 => unpack_constant::<2, UPSAMPLE>,
       4 => unpack_constant::<4, UPSAMPLE>,
       _ => unreachable!()
    })(slice, padding, callback);
}

#[inline(always)]
pub fn unpack_constant<const BITS: u8, const UPSAMPLE: bool>(slice: &[u8], padding: u8, mut callback: impl FnMut(&[u8])) {
    let mut i = 0; loop {
        let b = slice[i] as usize;

        let bytes = if UPSAMPLE { match BITS {
            1 => {UPSAMPLE_1BIT[b].as_slice()},
            2 => {UPSAMPLE_2BIT[b].as_slice()},
            4 => {UPSAMPLE_4BIT[b].as_slice()},
            _ => unreachable!()
        } } else { match BITS {
            1 => {UNPACK_1BIT[b].as_slice()},
            2 => {UNPACK_2BIT[b].as_slice()},
            4 => {UNPACK_4BIT[b].as_slice()},
            _ => unreachable!()
        } };

        if i == slice.len() - 1 {
            callback(&bytes[..bytes.len() - (padding/BITS) as usize]);

            break;
        }

        callback(bytes);

        i += 1;
    }
}

#[allow(clippy::cast_possible_truncation)]
const fn make_unpack_lut<const BITS: usize, const SAMPLES: usize, const UPSAMPLE: bool>() -> [[u8; SAMPLES]; 256] {
    let mut lut = [[0u8; SAMPLES]; 256];

    let mut byte = 0;
    while byte < 256 {
        let mut i = 0;
        while i < SAMPLES {
            let shift = 8 - BITS * (i + 1);
            let sample = (byte >> shift) & ((1 << BITS) - 1);

            // Expand to 8-bit range
            lut[byte][i] = if UPSAMPLE {(sample * 255 / ((1 << BITS) - 1)) as u8} else {sample as u8};

            i += 1;
        }
        byte += 1;
    }

    lut
}

pub const UPSAMPLE_1BIT: [[u8; 8]; 256] = make_unpack_lut::<1, 8, true>();
pub const UPSAMPLE_2BIT: [[u8; 4]; 256] = make_unpack_lut::<2, 4, true>();
pub const UPSAMPLE_4BIT: [[u8; 2]; 256] = make_unpack_lut::<4, 2, true>();

pub const UNPACK_1BIT: [[u8; 8]; 256] = make_unpack_lut::<1, 8, false>();
pub const UNPACK_2BIT: [[u8; 4]; 256] = make_unpack_lut::<2, 4, false>();
pub const UNPACK_4BIT: [[u8; 2]; 256] = make_unpack_lut::<4, 2, false>();
