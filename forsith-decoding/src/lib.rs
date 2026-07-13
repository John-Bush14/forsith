#![feature(portable_simd)]
#![feature(integer_casts)]

use std::{io::{self, Read}, ops::{BitAnd, BitOr, BitXor, Index, IndexMut, Range, Shl, Shr}};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use thiserror::Error;

mod tests;

mod png;
pub use png::PngDecoder;


use crate::png::ChunkType;

#[repr(u8)]
#[derive(TryFromPrimitive, IntoPrimitive)]
pub enum PixelFormat {
    Grayscale = 1,
    Truecolor = 3,
    GrayscaleAlpha = 2,
    TruecolorAlpha = 4
}

pub trait ImageDecoder<'a, R: Read, const D: u8, const F: u8> {
    fn dest_bitspp(&self) -> u8 {D * F}
    fn dest_bytespp(&self) -> u8 {D*F / 8}

    fn open(data: R) -> Result<Self, DecodingError> where Self: Sized;

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, DecodingError>;

    fn bit_depth(&self) -> u8;
    fn pixel_format(&self) -> PixelFormat;
}
impl<R: Read, const D: u8, const F: u8> Read for dyn ImageDecoder<'_, R, D, F> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {self.read(buf).map_err(io::Error::other)}
}

#[derive(Error, Debug)]
pub enum DecodingError {
    #[error("Incorrect header ({0:?})")]
    InccorectHeader(Vec<u8>),
    #[error("Unexpected IO error while reading data")]
    IOError(#[from] std::io::Error),

    // PNG specific
    #[error("Critical chunk '{0}' contains invalid data")]
    InvalidChunk(png::ChunkType),
    #[error("Interlacing is not supported")]
    InteralacingNotSupported,
    #[error("Unknown critical chunk type '{0:?}'")]
    UnkownChunk([u8; 4]),
    #[error("Stored ({0:?}) and calculated ({1:?}) CRC did not match, indicating data corruption.")]
    CRCMismatch(png::CRC32, png::CRC32), // calculated, store
    #[error("Stored ({1:#010X}) and calculated ({0:#010X}) Adler32 checksum did not match, indicating incorrect (de)compression.")]
    Adler32Mismatch(u32, u32), // calculated, store
    #[error("First chunk is not IHDR, instead ({0:?})")]
    NoIHDR(ChunkType),
    #[error("No IDAT chunk found")]
    NoIDAT,
    #[error("Multiple '{0}' chunks found")]
    MultipleChunks(ChunkType),
    #[error("Attempted to close chunk '{0}' with incorrect amount of bytes ({1}) remaining")]
    IncorrectClose(ChunkType, usize),
    #[error("Block length ({0}) and its one's complement ({1}) did not match")]
    BlockLengthMismatch(u16, u16),
    #[error("Code length ({0}) is too large")]
    InvalidCodeLength(u8),
    #[error("Tried to register huffman symbol with value larger than {0} bytes can hold.")]
    InvalidSymbol(usize),
    #[error("Undefined huffman code ({0:#010X}) found in deflate stream.")]
    UndefinedHuffmanCode(u32),
    #[error("Reserved compression method found in deflate stream.")]
    ReservedCompressionMethod,
    #[error("Invalid filter ({0}) written at start of scanline.")]
    InvalidFilter(u8),
    #[error("Invalid backreference with distance 0 found in deflate stream.")]
    ZeroDistance,
    #[error("Invalid bytes per pixel ({0}) calculated for image.")]
    InvalidStride(usize),
}
impl From<DecodingError> for io::Error {
    fn from(err: DecodingError) -> Self {
        io::Error::other(err)
    }
}

fn read_exact_array<const N: usize, R: Read>(reader: &mut R) -> io::Result<[u8; N]> {
    let mut buf = [0u8; N];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}
pub trait Num: Sized + Copy + Default + PartialEq
+ Eq + std::fmt::Debug + TryFrom<u32> + From<u8> + TryFrom<u16> + TryFrom<usize>
+ BitAnd<Output=Self> + BitOr<Output=Self> + BitXor<Output=Self> + Shl<usize, Output=Self>
+ Shr<usize, Output=Self> + std::ops::Add<Output=Self> + std::ops::Sub<Output=Self>
+ std::ops::Div<Output=Self> + std::ops::Mul<Output=Self> {
    fn read_be<R: Read>(reader: &mut R) -> io::Result<Self> where Self: Sized;
    fn read_le<R: Read>(reader: &mut R) -> io::Result<Self> where Self: Sized;
    const BIT_DEPTH: u8;
    const MAX: Self;
}
impl Num for u32 {
    fn read_be<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(u32::from_be_bytes(read_exact_array::<4, _>(reader)?))
    }
    fn read_le<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(u32::from_le_bytes(read_exact_array::<4, _>(reader)?))
    }
    const BIT_DEPTH: u8 = std::mem::size_of::<Self>() as u8 * 8;
    const MAX: Self = Self::MAX;
}
impl Num for u16 {
    fn read_be<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(u16::from_be_bytes(read_exact_array::<2, _>(reader)?))
    }
    fn read_le<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(u16::from_le_bytes(read_exact_array::<2, _>(reader)?))
    }
    const BIT_DEPTH: u8 = std::mem::size_of::<Self>() as u8 * 8;
    const MAX: Self = Self::MAX;
}
impl Num for u8 {
    fn read_be<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(u8::from_be_bytes(read_exact_array::<1, _>(reader)?))
    }
    fn read_le<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(u8::from_le_bytes(read_exact_array::<1, _>(reader)?))
    }
    const BIT_DEPTH: u8 = std::mem::size_of::<Self>() as u8 * 8;
    const MAX: Self = Self::MAX;
}
impl Num for usize {
    fn read_be<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(usize::from_be_bytes(read_exact_array::<{std::mem::size_of::<Self>()}, R>(reader)?))
    }
    fn read_le<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(usize::from_le_bytes(read_exact_array::<{std::mem::size_of::<Self>()}, R>(reader)?))
    }
    const BIT_DEPTH: u8 = std::mem::size_of::<Self>() as u8 * 8;
    const MAX: Self = Self::MAX;
}

#[derive(Debug)]
pub struct BitBuffer<I: Num> {
    buf: I,
    bits_remaining: u8
}
impl<I: Num> BitBuffer<I> {
    fn new() -> Self {
        Self {
            buf: I::default(),
            bits_remaining: 0
        }
    }

    pub fn bits_remaining(&self) -> u8 {
        self.bits_remaining
    }

    fn peek(&self, n: u8) -> I {
        if n > I::BIT_DEPTH {
            panic!("Cannot peek more than {} bits from this BitBuffer", I::BIT_DEPTH);
        }

        self.buf & ((I::from(1u8) << n as usize) - I::from(1u8))
    }

    fn consume(&mut self, n: u8) {
        self.buf = self.buf >> n as usize;
        self.bits_remaining -= n;
    }

    fn push(&mut self, byte: u8) {
        self.buf = self.buf | (I::from(byte) << self.bits_remaining as usize);
        self.bits_remaining += 8;
    }

    fn push_u32(&mut self, value: u32) { unsafe {
        self.buf = self.buf | (I::try_from(value).unwrap_unchecked() << self.bits_remaining as usize);
        self.bits_remaining += 32;
    }}
}

pub struct DestinationBuffer<'a, const D: u8, const F: u8> {
    buffer: &'a mut [u8],
    index: usize,
    full: bool
}

impl<'a, const D: u8, const F: u8> DestinationBuffer<'a, D, F> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self {
            buffer,
            index: 0,
            full: false
        }
    }

    pub fn push_byte(&mut self, b: u8) {
        unsafe {*self.buffer.get_unchecked_mut(self.index) = b};
        self.index += 1;
    }

    fn push_slice(&mut self, slice: &[u8]) {
        let len = slice.len();
        unsafe {self.buffer.get_unchecked_mut(self.index..self.index + len).copy_from_slice(slice)};
        self.index += len;
    }

    pub fn len(&self) -> usize {self.index}

    pub fn is_full(&self) -> bool {self.full}
    pub fn set_full(&mut self) {self.full = true;}

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn capacity(&self) -> usize {self.buffer.len()}
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

    pub fn push(&mut self, b: T) {
        unsafe {*self.buffer.get_unchecked_mut(self.cursor) = b};
        self.cursor += 1;
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
pub struct BufferReader {
    buffer: Vec<u8>,
    index: usize
}

impl BufferReader {
    pub fn new(len: usize) -> Self {
        Self {
            buffer: vec![0u8; len],
            index: 0
        }
    }

    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }

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

    pub fn read_be<N: Num>(&mut self) -> N {
        let value = N::read_be(&mut &self.buffer[self.index..]).unwrap();
        self.index += std::mem::size_of::<N>();
        value
    }

    pub fn read_le<N: Num>(&mut self) -> N {
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
        self.buffer.len() - self.index
    }
}
