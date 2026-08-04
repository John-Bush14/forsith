use std::{fmt::Debug, io::Read, marker::PhantomData, ops::Range};
use crate::{Channel, Int, decompression::BitReader};
use derive_more::{Deref, DerefMut, Index, IndexMut};

#[derive(Debug, Default)]
pub struct BitBuffer {
    buf: u64,
    bits_remaining: u8
}
impl BitBuffer {
    #[inline(always)]
    pub fn bits_remaining(&self) -> u8 {
        self.bits_remaining
    }

    #[inline(always)]
    pub fn peek(&self, n: u8) -> u64 {
        #[cfg(debug_assertions)]
        if n > 64 {
            panic!("Cannot peek more than {} bits from this BitBuffer", 64);
        }

        self.buf & ((1 << n as usize) - 1)
    }

    #[inline(always)]
    pub fn consume(&mut self, n: u8) {
        self.buf >>= n as usize;
        self.bits_remaining -= n;
    }

    #[inline(always)]
    pub fn push<T: Int>(&mut self, value: T) {
        assert!(T::MIN == 0, "BitBuffer.push should only be called with unsigned ints");

        let value: u64 = match value.try_into() {Ok(v) => v, _ => unreachable!()};

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
    pub fn new(buffer: &'a mut [C::StorageType]) -> Self {
        Self {
            buffer,
            index: 0,
            full: false,
            stride: 1,
            _phantom: Default::default()
        }
    }

    #[inline(always)]
    pub fn push_channel(&mut self, c: C::StorageType) {
        self.buffer[self.index] = c;
        self.index += 1;
    }

    pub fn remaining(&self) -> usize {self.buffer.len().saturating_sub(self.index)}
    #[inline(always)]
    pub fn remaining_bytes(&self) -> usize {self.remaining() * C::StorageType::BYTE_DEPTH as usize}

    pub fn set_stride(&mut self, pixels: usize) {self.stride = (pixels - 1) * F as usize;}
    #[inline(always)]
    pub fn pushed_pixel(&mut self) {self.index += self.stride}
    #[inline(always)]
    pub fn advance(&mut self, pixels: usize) {
        self.index += pixels * F as usize;
    }
    pub fn reset(&mut self) {self.index = 0;}

    pub fn len(&self) -> usize {self.index}
    pub fn bytes_len(&self) -> usize {self.index * C::StorageType::BYTE_DEPTH as usize}

    pub fn is_full(&self) -> bool {self.full}
    pub fn set_full(&mut self) {self.full = true;}

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn capacity(&self) -> usize {self.buffer.len()}
    pub fn bytes_capacity(&self) -> usize {self.buffer.len() * C::StorageType::BYTE_DEPTH as usize}

    pub fn remaining_mut_slice(&mut self) -> &mut [C::StorageType] {&mut self.buffer[self.index..]}
}

#[derive(Debug, Index, IndexMut, Default)]
pub struct CursorVec<T> {
    #[index]
    #[index_mut]
    buffer: Vec<T>,
    cursor: usize,
}

impl<T> CursorVec<T> {
    pub fn new(size: usize) -> Self where T: Default + Copy {
        Self {
            buffer: vec![T::default(); size],
            cursor: 0
        }
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.buffer.as_mut_ptr()
    }

    #[inline(always)]
    pub fn push(&mut self, b: T) {
        self.buffer[self.cursor] = b;
        self.cursor += 1;
    }

    pub fn push_slice(&mut self, slice: &[T]) where T: Copy {
        let len = slice.len();
        self.mut_slice(self.cursor..self.cursor + len).copy_from_slice(slice);
        self.cursor += len;
    }

    pub fn slice(&self, range: Range<usize>) -> &[T] {&self.buffer[range]}

    pub fn mut_slice(&mut self, range: Range<usize>) -> &mut [T] {&mut self.buffer[range]}

    pub fn copy_within(&mut self, src: Range<usize>, dest: usize) where T: Copy {
        self.buffer.copy_within(src, dest);
    }

    #[inline(always)]
    pub fn advance(&mut self, n: usize) {
        self.cursor += n;
    }

    pub fn set_cursor(&mut self, i: usize) {self.cursor = i}

    pub fn clear(&mut self) {
        self.cursor = 0;
    }

    pub fn full_buf_slice(&self) -> &[T] {self.buffer.as_slice()}

    pub fn as_slice(&self) -> &[T] where T: Debug {
        self.slice(0..self.cursor)
    }

    pub fn len(&self) -> usize {self.cursor}
    pub fn capacity(&self) -> usize {self.buffer.len()}
    pub fn remaining(&self) -> usize {self.capacity() - self.len()}

    #[must_use]
    pub fn is_empty(&self) -> bool {self.len() == 0}

    #[must_use]
    pub fn is_full(&self) -> bool {self.len() == self.capacity()}
}
impl CursorVec<u8> {
    pub fn read_from<R: Read>(&mut self, reader: &mut R, len: usize) -> std::io::Result<()> {
        self.cursor += len;
        let buf = self.mut_slice(self.cursor - len..self.cursor);
        reader.read_exact(buf)
    }
}

#[derive(Debug, Default, Deref, DerefMut)]
pub struct BitBufferReader {
    #[deref]
    #[deref_mut]
    buffer: BufferReader,
    bit_buf: BitBuffer,
}
impl BitBufferReader {
    pub fn new(start_len: usize) -> Self {
        Self {
            buffer: BufferReader::new(start_len),
            bit_buf: BitBuffer::default(),
        }
    }

    pub fn align(&mut self) -> Result<(), std::io::Error> {
        let alignment = 4 - (self.buffer.cursor % align_of::<u32>());

        let mut buf = vec![0u8; alignment];
        self.buffer.read_exact(&mut buf)?;

        for b in buf {self.bit_buf.push(b)}; Ok(())
    }

    pub fn unconsume_bitbuf(&mut self) {
        let bitbuf_bytes = self.bit_buf.bits_remaining().div_euclid(8) as usize;

        self.buffer.unconsume(bitbuf_bytes);
        self.bit_buf.consume(self.bit_buf.bits_remaining());
    }
}

impl BitReader for BitBufferReader {
    #[inline(always)]
    fn peek_bits(&mut self, n: u8) -> u64 {
        if self.bit_buf.bits_remaining() <= 32 {
            self.fill_bitbuf();
        }

        self.bit_buf.peek(n)
    }

    #[inline(always)]
    fn fill_bitbuf(&mut self) {
        let refil = u32::from_le_bytes(self.buffer.buffer[self.buffer.cursor..self.buffer.cursor + 4].try_into().unwrap());
        self.buffer.consume(4);

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
impl Read for BitBufferReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.buffer.read(buf)
    }
}

#[derive(Debug, Default)]
pub struct BufferReader {
    buffer: Vec<u8>,
    cursor: usize,
}
impl BufferReader {
    pub fn new(start_len: usize) -> Self {
        Self {
            buffer: vec![0u8; start_len],
            cursor: 0,
        }
    }

    pub fn capacity(&self) -> usize {self.buffer.len()}

    pub fn mut_buffer(&mut self) -> &mut Vec<u8> {&mut self.buffer}

    pub fn slice(&self, len: usize) -> &[u8] {
        &self.buffer[self.cursor..self.cursor + len]
    }

    pub fn mut_slice(&mut self, len: usize) -> &mut [u8] {
        &mut self.buffer[self.cursor..self.cursor + len]
    }

    pub fn raw_slice(&self, range: Range<usize>) -> &[u8] {&self.buffer[range]}
    pub fn raw_mut_slice(&mut self, range: Range<usize>) -> &mut [u8] {&mut self.buffer[range]}

    #[inline(always)]
    pub fn consume(&mut self, n: usize) {self.cursor += n;}
    pub fn unconsume(&mut self, n: usize) {self.cursor -= n;}

    pub fn expand(&mut self, len: usize) {self.buffer.resize(self.buffer.len() + len, 0u8)}

    pub fn remaining(&self) -> usize {
        self.buffer.len() - self.cursor
    }

    pub fn shrink_buffer(&mut self, len: usize) {
        self.buffer.truncate(len);
        self.buffer.shrink_to_fit();
    }

    pub fn cursor(&self) -> usize {self.cursor}
    pub fn set_cursor(&mut self, cursor: usize) {self.cursor = cursor;}
}

impl Read for BufferReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        buf.copy_from_slice(&self.buffer[self.cursor..self.cursor+buf.len()]);
        self.cursor += buf.len();

        Ok(buf.len())
    }
}
