use std::{fmt::Debug, io::{BufRead, Cursor, Read, Seek}, marker::PhantomData, ops::Deref};
use crate::{int::Int, parsing::BitRead};
use derive_more::{Deref, DerefMut};

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

#[derive(Debug, Deref, DerefMut)]
pub struct CursorVec<T>(Cursor<Vec<T>>);

impl<T: Default + Clone> CursorVec<T> {
    pub fn new(len: usize) -> Self {Self(Cursor::new(vec![T::default(); len]))}

    pub fn expand(&mut self, len: usize) {
        let cap = self.capacity();
        self.get_mut().resize(cap + len, T::default());
    }
}

impl<T> Default for CursorVec<T> {fn default() -> Self {Self(Cursor::new(Vec::new()))}}

impl<T> CursorVec<T> {
    pub fn into_inner(self) -> Cursor<Vec<T>> {self.0}
    pub fn read_single(&mut self) -> &T {&self.take_slice(1)[0]}
    pub fn remaining(&self) -> usize {self.capacity() - self.cursor()}
    pub fn capacity(&self) -> usize {self.get_ref().len()}
    pub fn cursor(&self) -> usize {usize::try_from(self.position()).unwrap()}
    pub fn set_cursor(&mut self, cursor: usize) {self.set_position(cursor as u64);}
    pub fn consume(&mut self, len: usize) {self.set_cursor(self.cursor() + len);}
    pub fn unconsume(&mut self, len: usize) {self.set_cursor(self.cursor().saturating_sub(len));}
    #[must_use]
    pub fn is_empty(&self) -> bool {self.capacity() == 0}
    pub fn is_full(&self) -> bool {self.cursor() == self.capacity()}
    pub fn current(&self) -> Option<&T> {self.get_ref().get(self.cursor())}

    #[inline(always)]
    pub fn write_fast_single(&mut self, data: T) {
        let cursor = self.cursor();
        self.get_mut()[cursor] = data;
        self.set_cursor(cursor + 1);
    }

    pub fn take_slice(&mut self, len: usize) -> &[T] {
        let cursor = self.cursor();
        self.set_cursor(cursor + len);
        &self.get_ref()[cursor..cursor + len]
    }
    pub fn take_mut_slice(&mut self, len: usize) -> &mut [T] {
        let cursor = self.cursor();
        self.set_cursor(cursor + len);
        &mut self.get_mut()[cursor..cursor + len]
    }
}
impl CursorVec<u8> {
    pub fn fill_from(&mut self, reader: &mut impl Read, len: usize) -> std::io::Result<()> {
        let cursor = self.cursor();
        let buf = &mut self.0.get_mut()[cursor..cursor + len];
        reader.read_exact(buf)?;

        Ok(())
    }
    pub fn read_from(&mut self, reader: &mut impl Read, len: usize) -> std::io::Result<()> {
        self.fill_from(reader, len)?;
        self.consume(len);
        Ok(())
    }
}

impl<T> From<Vec<T>> for CursorVec<T> where T: Default + Clone {
    fn from(vec: Vec<T>) -> Self {
        Self(Cursor::new(vec))
    }
}

impl Read for CursorVec<u8> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {(**self).read(buf)}
}

impl Seek for CursorVec<u8> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {(**self).seek(pos)}
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
    fn fill_bitbuf(&mut self) {
        // doesn't use read_le for performance reasons
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
        let refill = self.read_le::<u32>().unwrap();
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
