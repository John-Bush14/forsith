use std::io::{Read, Seek};

use forsith_proc::{Deref, DerefMut};

use crate::{buffers::CursorVec, int::Int};

pub trait BitRead {
    fn fill_bitbuf(&mut self);
    fn peek_bits(&mut self, n: u8) -> u64;
    fn peek_bits_nobranch(&mut self, n: u8) -> u64;
    fn consume_bits(&mut self, n: u8);
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
    fn iterate_bits<const BITS: u8>(&mut self) -> BitIterator<'_, Self, BITS>
    where
        Self: Sized,
    {
        BitIterator { reader: self }
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
    bits_remaining: u8,
}
impl BitBuffer {
    #[inline(always)]
    #[must_use]
    pub const fn bits_remaining(&self) -> u8 {
        self.bits_remaining
    }

    #[inline(always)]
    #[must_use]
    pub const fn peek(&self, n: u8) -> u64 {
        self.buf & ((1 << n as usize) - 1)
    }

    #[inline(always)]
    pub const fn consume(&mut self, n: u8) {
        self.buf >>= n as usize;
        self.bits_remaining -= n;
    }

    #[inline(always)]
    #[allow(clippy::missing_panics_doc)] // constant assertion, will never (or always) panic
    pub fn push<T: Int>(&mut self, value: T) {
        assert!(
            T::MIN == 0,
            "BitBuffer.push should only be called with unsigned ints"
        );

        let value: u64 = value.try_into().unwrap_or_else(|_| unreachable!());

        self.buf |= value << self.bits_remaining as usize;
        self.bits_remaining += T::BIT_DEPTH;
    }
}

#[derive(Debug, Deref, DerefMut)]
pub struct BitReader<T: Read + Seek> {
    #[deref]
    #[deref_mut]
    buffer: T,
    bit_buf: BitBuffer,
}

impl<T: Read + Seek> BitReader<T> {
    pub fn new(buffer: T) -> Self {
        Self {
            buffer,
            bit_buf: BitBuffer::default(),
        }
    }

    /// # Panics
    /// Panics if `seek_relative` fails
    pub fn unconsume_bitbuf(&mut self) {
        let bitbuf_bytes = self.bit_buf.bits_remaining().div_euclid(8);

        self.buffer.seek_relative(-i64::from(bitbuf_bytes)).unwrap();
        self.bit_buf.consume(self.bit_buf.bits_remaining());
    }
}

impl BitReader<CursorVec<u8>> {
    #[inline(always)]
    #[allow(clippy::missing_panics_doc)] // Slice will always be 4 bytes, so this will never panic
    pub fn fill_bitbuf(&mut self) {
        // performance reasons
        let refil = u32::from_le_bytes(self.buffer.take_mut_slice(4).try_into().unwrap());

        self.bit_buf.push(refil);
    }
}

impl<T: Read + Seek> BitRead for BitReader<T> {
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

    #[inline(always)]
    fn peek_bits_nobranch(&mut self, n: u8) -> u64 {
        self.bit_buf.peek(n)
    }
}

pub fn unpack<const UPSAMPLE: bool>(
    slice: &[u8],
    bits: u8,
    padding: u8,
    callback: impl FnMut(&[u8]),
) {
    (match bits {
        1 => unpack_constant::<1, UPSAMPLE>,
        2 => unpack_constant::<2, UPSAMPLE>,
        4 => unpack_constant::<4, UPSAMPLE>,
        _ => unreachable!(),
    })(slice, padding, callback);
}

#[inline(always)]
pub fn unpack_constant<const BITS: u8, const UPSAMPLE: bool>(
    slice: &[u8],
    padding: u8,
    mut callback: impl FnMut(&[u8]),
) {
    let mut i = 0;
    loop {
        let b = slice[i] as usize;

        let bytes = if UPSAMPLE {
            match BITS {
                1 => UPSAMPLE_1BIT[b].as_slice(),
                2 => UPSAMPLE_2BIT[b].as_slice(),
                4 => UPSAMPLE_4BIT[b].as_slice(),
                _ => unreachable!(),
            }
        } else {
            match BITS {
                1 => UNPACK_1BIT[b].as_slice(),
                2 => UNPACK_2BIT[b].as_slice(),
                4 => UNPACK_4BIT[b].as_slice(),
                _ => unreachable!(),
            }
        };

        if i == slice.len() - 1 {
            callback(&bytes[..bytes.len() - (padding / BITS) as usize]);

            break;
        }

        callback(bytes);

        i += 1;
    }
}

#[allow(clippy::cast_possible_truncation)]
const fn make_unpack_lut<const BITS: usize, const SAMPLES: usize, const UPSAMPLE: bool>()
-> [[u8; SAMPLES]; 256] {
    let mut lut = [[0u8; SAMPLES]; 256];

    let mut byte = 0;
    while byte < 256 {
        let mut i = 0;
        while i < SAMPLES {
            let shift = 8 - BITS * (i + 1);
            let sample = (byte >> shift) & ((1 << BITS) - 1);

            // Expand to 8-bit range
            lut[byte][i] = if UPSAMPLE {
                (sample * 255 / ((1 << BITS) - 1)) as u8
            } else {
                sample as u8
            };

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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_unpack<const BITS: u8, const UPSAMPLE: bool>(
        input: &[u8],
        expected: &[u8],
        padding: u8,
    ) {
        let mut output = Vec::new();

        unpack::<UPSAMPLE>(input, BITS, padding, |chunk| {
            output.extend_from_slice(chunk)
        });

        assert_eq!(output, expected);
    }

    #[test]
    fn test_unpack_1bit() {
        test_unpack::<1, false>(
            &[0b1010_1010, 0b1100_1100],
            &[1, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 0],
            4,
        );
    }

    #[test]
    fn test_unpack_1bit_upsample() {
        test_unpack::<1, true>(
            &[0b1010_1010, 0b1100_1100],
            &[255, 0, 255, 0, 255, 0, 255, 0, 255, 255, 0, 0, 255, 255],
            2,
        );
    }

    #[test]
    fn test_unpack_2bit() {
        test_unpack::<2, false>(&[0b1100_0011, 0b1010_0101], &[3, 0, 0, 3, 2, 2, 1, 1], 0);
    }

    #[test]
    fn test_unpack_2bit_upsample() {
        test_unpack::<2, true>(
            &[0b1100_0011, 0b1010_0101],
            &[255, 0, 0, 255, 170, 170, 85, 85],
            0,
        );
    }

    #[test]
    fn test_unpack_4bit() {
        test_unpack::<4, false>(&[0b1010_1010, 0b1100_1100], &[10, 10, 12, 12], 0);
    }

    #[test]
    fn test_unpack_4bit_upsample() {
        test_unpack::<4, true>(&[0b1010_1010, 0b1100_1100], &[170, 170, 204, 204], 0);
    }

    #[test]
    fn test_bit_buffer_push_and_consume() {
        let mut bit_buf = BitBuffer::default();

        bit_buf.push(0b1010_1010u8);
        assert_eq!(bit_buf.bits_remaining(), 8);
        assert_eq!(bit_buf.peek(8), 0b1010_1010);

        bit_buf.consume(4);
        assert_eq!(bit_buf.bits_remaining(), 4);
        assert_eq!(bit_buf.peek(4), 0b1010);

        bit_buf.consume(4);
        assert_eq!(bit_buf.bits_remaining(), 0);
        assert_eq!(bit_buf.peek(1), 0);
    }

    #[test]
    fn test_bit_reader_reads_bits_in_sequence() {
        let mut reader = BitReader::new(std::io::Cursor::new(vec![
            0b1100_0011,
            0b1010_0101,
            0b1010_0101,
            0b1010_0101,
            0,
            0,
            0,
            0, // padding to ensure we have enough bytes
        ]));

        assert_eq!(reader.read_bits(2), 0b11);
        assert_eq!(reader.read_bits(2), 0b00);
        assert_eq!(reader.read_bits(2), 0b00);
        assert_eq!(reader.read_bits(2), 0b11);
        assert_eq!(reader.read_bits(4), 0b0101);
        assert_eq!(reader.read_bits(4), 0b1010);
        assert_eq!(reader.read_bits(8), 0b1010_0101);
        assert_eq!(reader.read_bits(8), 0b1010_0101);
    }

    #[test]
    fn test_bit_reader_unconsume_bitbuf() {
        let mut reader = BitReader::new(std::io::Cursor::new(vec![
            0b1100_0011,
            0b1010_0101,
            0b1010_0101,
            0b1010_0101,
            0,
            0,
            0,
            0, // padding to ensure we have enough bytes
        ]));

        reader.read_bits(16);

        reader.unconsume_bitbuf();
        let mut rest = [0u8; 6];
        reader.read_exact(&mut rest).unwrap();

        assert_eq!(rest, [0b1010_0101, 0b1010_0101, 0, 0, 0, 0]);
    }

    fn test_bit_iterator<const BITS: u8>() {
        let bytes = vec![
            0b1100_0011,
            0b1010_0101,
            0b1010_0101,
            0b1010_0101,
            0,
            0,
            0,
            0,
        ];

        let mut reader = BitReader::new(std::io::Cursor::new(&bytes));
        let mut iter = reader.iterate_bits::<BITS>().take(32 / BITS as usize);

        unpack_constant::<BITS, false>(&bytes[..4], 0, |chunk| {
            for &bit in chunk.iter().rev() {
                assert_eq!(iter.next().unwrap(), bit.into());
            }
        });
    }

    #[test]
    fn test_1bit_iterator() {
        test_bit_iterator::<1>();
    }

    #[test]
    fn test_2bit_iterator() {
        test_bit_iterator::<2>();
    }

    #[test]
    fn test_4bit_iterator() {
        test_bit_iterator::<4>();
    }
}
