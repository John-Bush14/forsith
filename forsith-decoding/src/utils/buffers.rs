use std::{fmt::Debug, io::{Cursor, Read}, marker::PhantomData};
use crate::{Channel, Int, parsing::BitReader};
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

pub struct OutputWriter<'a, C: Channel, const F: u8> {
    buffer: &'a mut [C::StorageType],
    index: usize,
    full: bool,
    stride: usize,
    _phantom: PhantomData<C>
}

impl<'a, C: Channel, const F: u8> OutputWriter<'a, C, F> {
    pub const fn new(buffer: &'a mut [C::StorageType]) -> Self {
        Self {
            buffer,
            index: 0,
            full: false,
            stride: 1,
            _phantom: PhantomData
        }
    }

    #[inline(always)]
    pub const fn push_channel(&mut self, c: C::StorageType) {
        self.buffer[self.index] = c;
        self.index += 1;
    }

    pub const fn remaining(&self) -> usize {self.buffer.len().saturating_sub(self.index)}
    #[inline(always)]
    pub const fn remaining_bytes(&self) -> usize {self.remaining() * C::StorageType::BYTE_DEPTH as usize}

    pub const fn set_stride(&mut self, pixels: usize) {self.stride = (pixels - 1) * F as usize;}
    #[inline(always)]
    pub const fn pushed_pixel(&mut self) {self.index += self.stride}
    #[inline(always)]
    pub const fn advance(&mut self, pixels: usize) {
        self.index += pixels * F as usize;
    }
    pub const fn reset(&mut self) {self.index = 0;}

    pub const fn len(&self) -> usize {self.index}
    pub const fn bytes_len(&self) -> usize {self.index * C::StorageType::BYTE_DEPTH as usize}

    pub const fn is_full(&self) -> bool {self.full}
    pub const fn set_full(&mut self) {self.full = true;}

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub const fn capacity(&self) -> usize {self.buffer.len()}
    pub const fn bytes_capacity(&self) -> usize {self.buffer.len() * C::StorageType::BYTE_DEPTH as usize}

    pub fn remaining_mut_slice(&mut self) -> &mut [C::StorageType] {&mut self.buffer[self.index..]}
}

#[derive(Debug, Default, Deref, DerefMut)]
pub struct CursorVec<T: Default + Clone>(Cursor<Vec<T>>);
impl<T: Default + Clone> CursorVec<T> {
    pub fn new(len: usize) -> Self {Self(Cursor::new(vec![T::default(); len]))}

    pub fn remaining(&self) -> usize {self.capacity() - self.cursor()}
    pub fn capacity(&self) -> usize {self.get_ref().len()}
    pub fn cursor(&self) -> usize {usize::try_from(self.position()).unwrap()}
    pub fn set_cursor(&mut self, cursor: usize) {self.set_position(cursor as u64);}
    pub fn consume(&mut self, len: usize) {self.set_cursor(self.cursor() + len);}
    pub fn unconsume(&mut self, len: usize) {self.set_cursor(self.cursor().saturating_sub(len));}
    pub fn is_empty(&self) -> bool {self.cursor() == 0}
    pub fn is_full(&self) -> bool {self.cursor() == self.capacity()}

    #[inline(always)]
    pub fn write_fast_single(&mut self, data: T) {
        let cursor = self.cursor();
        self.get_mut()[cursor] = data;
        self.set_cursor(cursor + 1);
    }

    pub fn expand(&mut self, len: usize) {
        let cap = self.capacity();
        self.get_mut().resize(cap + len, T::default());
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

#[derive(Debug, Default, Deref, DerefMut)]
pub struct BitCursorVec {
    #[deref]
    #[deref_mut]
    buffer: CursorVec<u8>,
    bit_buf: BitBuffer,
}
impl BitCursorVec {
    pub fn new(start_len: usize) -> Self {
        Self {
            buffer: CursorVec::<u8>::new(start_len),
            bit_buf: BitBuffer::default(),
        }
    }

    pub fn align(&mut self) -> Result<(), std::io::Error> {
        let alignment = 4 - (self.buffer.cursor() % align_of::<u32>());

        let mut buf = vec![0u8; alignment];
        self.buffer.read_exact(&mut buf)?;

        for b in buf {self.bit_buf.push(b)}; Ok(())
    }

    pub fn unconsume_bitbuf(&mut self) {
        let bitbuf_bytes = self.bit_buf.bits_remaining().div_euclid(8);

        self.buffer.unconsume(bitbuf_bytes as _);
        self.bit_buf.consume(self.bit_buf.bits_remaining());
    }
}

impl BitReader for BitCursorVec {
    #[inline(always)]
    fn peek_bits(&mut self, n: u8) -> u64 {
        if self.bit_buf.bits_remaining() <= 32 {
            self.fill_bitbuf();
        }

        self.bit_buf.peek(n)
    }

    #[inline(always)]
    fn fill_bitbuf(&mut self) {
        // doesn't use read_le for performance reasons
        let refil = u32::from_le_bytes(self.buffer.take_mut_slice(4).try_into().unwrap());

        self.bit_buf.push(refil);
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
