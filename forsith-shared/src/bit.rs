use std::io::{Read, Seek};

use derive_more::{Deref, DerefMut};

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

#[derive(Debug, Default, Deref, DerefMut)]
pub struct BitReader<T: Read + Default + Seek> {
    #[deref]
    #[deref_mut]
    buffer: T,
    bit_buf: BitBuffer,
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

