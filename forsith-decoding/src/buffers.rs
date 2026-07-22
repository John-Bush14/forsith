use std::{marker::PhantomData, ops::{Index, IndexMut, Range}};

#[derive(Debug)]
pub struct BitBuffer {
    buf: u64,
    bits_remaining: u8
}
impl BitBuffer {
    pub fn bits_remaining(&self) -> u8 {
        self.bits_remaining
    }

    pub fn peek(&self, n: u8) -> u64 {
        if n > 64 {
            panic!("Cannot peek more than {} bits from this BitBuffer", 64);
        }

        self.buf & ((1 << n as usize) - 1)
    }

    pub fn consume(&mut self, n: u8) {
        self.buf = self.buf >> n as usize;
        self.bits_remaining -= n;
    }

    pub fn push_u32(&mut self, value: u32) {
        self.buf = self.buf | ((value as u64) << self.bits_remaining as usize);
        self.bits_remaining += 32;
    }
}
impl Default for BitBuffer {
    fn default() -> Self {
        Self {
            buf: 0,
            bits_remaining: 0
        }
    }
}

pub struct OutputWriter<'a, C: Channel> {
    buffer: &'a mut [u8],
    index: usize,
    full: bool,
    stride: usize,
    _phantom: PhantomData<C>
}

impl<'a, C: Channel> OutputWriter<'a, C> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
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
        #[cfg(debug_assertions)]
        if (self.buffer.len() - self.index) < C::BIT_DEPTH as usize/8 {panic!("tried to push channel into full dest")}

        unsafe {self.channel_ptr().write(c)};

        self.index += self.stride;
    }

    fn channel_ptr(&mut self) -> *mut C::StorageType {
        #[cfg(debug_assertions)]
        if self.buffer.as_mut_ptr().wrapping_add(self.index).is_null() {panic!("channel ptr null!");}

        self.buffer.as_mut_ptr().wrapping_add(self.index) as *mut C::StorageType
    }

    fn bbp() -> usize {const {C::BIT_DEPTH as usize / 8}}

    pub fn set_stride(&mut self, pixels: usize) {self.stride = pixels * Self::bbp()}
    pub fn advance(&mut self, pixels: usize) {self.index += pixels * Self::bbp()}

    pub fn len(&self) -> usize {self.index}

    pub fn is_full(&self) -> bool {self.full}
    pub fn set_full(&mut self) {self.full = true;}

    #[must_use]
    #[allow(unused)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn capacity(&self) -> usize {self.buffer.len()}

    pub fn remaining_mut_slice(&mut self) -> &mut [u8] {&mut self.buffer[self.index..]}
}

#[derive(Debug)]
pub struct CursorVec<T> {
    buffer: Vec<T>,
    cursor: usize,
}

impl<T> Default for CursorVec<T> where T: Default + Copy {
    fn default() -> Self {
        Self::new(0)
    }
}

impl<T> Index<usize> for CursorVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe {self.buffer.get_unchecked(index)}
    }
}
impl<T> IndexMut<usize> for CursorVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe {self.buffer.get_unchecked_mut(index)}
    }
}

impl<T> CursorVec<T> {
    pub fn new(size: usize) -> Self where T: Default + Copy {
        Self {
            buffer: vec![T::default(); size],
            cursor: 0
        }
    }

    #[inline(always)]
    pub fn push(&mut self, b: T) {
        unsafe {
            #[cfg(debug_assertions)]
            if self.buffer.as_ptr().add(self.cursor).is_null() {panic!("null cursorvec ptr")}

            *self.buffer.as_mut_ptr().add(self.cursor) = b;
            self.cursor = self.cursor.unchecked_add(1);
        }
    }

    pub fn push_slice(&mut self, slice: &[T]) where T: Copy {
        let len = slice.len();
        unsafe {self.buffer.get_unchecked_mut(self.cursor..self.cursor + len).copy_from_slice(slice)};
        self.cursor += len;
    }

    pub fn slice(&self, range: Range<usize>) -> &[T] {
        unsafe {self.buffer.get_unchecked(range)}
    }

    pub fn mut_slice(&mut self, range: Range<usize>) -> &mut [T] {
        unsafe {self.buffer.get_unchecked_mut(range)}
    }

    pub fn copy_within(&mut self, src: Range<usize>, dest: usize) where T: Copy {
        self.buffer.copy_within(src, dest);
    }

    pub fn advance(&mut self, n: usize) {
        self.cursor += n;
    }

    pub fn shift(&mut self, new_start: usize) where T: Copy {
        self.buffer.copy_within(new_start..self.cursor, 0);

        self.cursor -= new_start;
    }

    pub fn clear(&mut self) {
        self.cursor = 0;
    }

    pub fn full_buf_slice(&self) -> &[T] {self.buffer.as_slice()}

    pub fn as_slice(&self) -> &[T] {
        unsafe {self.buffer.get_unchecked(..self.cursor)}
    }

    pub fn len(&self) -> usize {self.cursor}
    pub fn capacity(&self) -> usize {self.buffer.len()}
    pub fn remaining(&self) -> usize {self.capacity() - self.len()}

    #[must_use]
    pub fn is_empty(&self) -> bool {self.len() == 0}

    #[must_use]
    pub fn is_full(&self) -> bool {self.len() == self.capacity()}
}

#[derive(Debug)]
pub struct BufferReader<const LEN: usize> {
    buffer: [u8; LEN],
    index: usize
}
impl<const LEN: usize> Default for BufferReader<LEN> {
    fn default() -> Self {
        Self {
            buffer: [0u8; LEN],
            index: 0
        }
    }
}
impl<const LEN: usize> BufferReader<LEN> {
    pub fn slice(&self, len: usize) -> &[u8] {
        unsafe {self.buffer.get_unchecked(self.index..self.index + len)}
    }

    pub fn mut_slice(&mut self, len: usize) -> &mut [u8] {
        unsafe {self.buffer.get_unchecked_mut(self.index..self.index + len)}
    }

    pub fn raw_slice(&self, range: Range<usize>) -> &[u8] {
        unsafe {self.buffer.get_unchecked(range)}
    }
    pub fn raw_mut_slice(&mut self, range: Range<usize>) -> &mut [u8] {
        unsafe {self.buffer.get_unchecked_mut(range)}
    }

    pub fn consume(&mut self, n: usize) {
        self.index += n;
    }

    pub fn unconsume(&mut self, n: usize) {
        self.index -= n;
    }

    pub fn read_be<N: Int>(&mut self) -> N {
        let value = N::read_be(&mut &self.buffer[self.index..]).unwrap();
        self.index += std::mem::size_of::<N>();
        value
    }

    pub fn read_le<N: Int>(&mut self) -> N {
        let value = N::read_le(&mut &self.buffer[self.index..]).unwrap();
        self.index += std::mem::size_of::<N>();
        value
    }

    pub fn read_array<const N: usize>(&mut self) -> [u8; N] {
        let value = self.slice(N).try_into().unwrap();
        self.index += N;
        value
    }

    pub fn empty(&mut self) {
        self.buffer.copy_within(self.index.., 0);
        self.index = 0;
    }

    pub fn remaining(&self) -> usize {
        LEN - self.index
    }
}
