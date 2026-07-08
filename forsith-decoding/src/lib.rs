use std::{io::{self, Read}, ops::Index};
use num_enum::{FromPrimitive, IntoPrimitive, TryFromPrimitive};
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

pub trait ImageDecoder<'a, R: Read, C: Num, const F: u8> {
    fn open(data: R) -> Result<Self, DecodingError> where Self: Sized;

    fn next(&mut self) -> Option<Result<&'a [u8], DecodingError>>;

    fn bit_depth(&self) -> u8;
    fn pixel_format(&self) -> PixelFormat;
}
impl<'a, R: Read, C: Num, const F: u8> Iterator for dyn ImageDecoder<'a, R, C, F> {
    type Item = Result<&'a [u8], DecodingError>;

    fn next(&mut self) -> Option<Self::Item> {<Self as ImageDecoder<'a, R, C, F>>::next(self)}
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
    #[error("Stored ({1:#010X}) and calculated ({0:#010X}) CRC did not match, indication data corruption.")]
    CRCMismatch(u32, u32), // calculated, store
    #[error("First chunk is not IHDR, instead ({0:?})")]
    NoIHDR(ChunkType),
    #[error("No IDAT chunk found")]
    NoIDAT,
    #[error("Multiple '{0}' chunks found")]
    MultipleChunks(ChunkType),
    #[error("Attempted to close chunk '{0}' before reading all data, {1} bytes remaining")]
    EarlyClose(ChunkType, u32),
    #[error("Block length ({0}) and its one's complement ({1}) did not match")]
    BlockLengthMismatch(u16, u16),
    #[error("Code length ({0}) is too large")]
    InvalidCodeLength(u8),
    #[error("Tried to register huffman symbol with value larger than {0} bytes can hold.")]
    InvalidSymbol(usize),
    #[error("Undefined huffman code ({0:#010X}) found in deflate stream.")]
    UndefinedHuffmanCode(u32),
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
pub trait Num: Sized + Copy + Default + PartialEq + Eq + std::fmt::Debug + TryFrom<u32> + From<u8> + TryFrom<u16> + TryFrom<usize> {
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
    const BIT_DEPTH: u8 = 32;
    const MAX: Self = Self::MAX;
}
impl Num for u16 {
    fn read_be<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(u16::from_be_bytes(read_exact_array::<2, _>(reader)?))
    }
    fn read_le<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(u16::from_le_bytes(read_exact_array::<2, _>(reader)?))
    }
    const BIT_DEPTH: u8 = 8;
    const MAX: Self = Self::MAX;
}
impl Num for u8 {
    fn read_be<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(u8::from_be_bytes(read_exact_array::<1, _>(reader)?))
    }
    fn read_le<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(u8::from_le_bytes(read_exact_array::<1, _>(reader)?))
    }
    const BIT_DEPTH: u8 = 8;
    const MAX: Self = Self::MAX;
}

/// Constant size buffer where the oldest element is overwritten when the buffer is full.
/// Index 0 is here the most recent element and Index len-1 is the oldest element.
///
/// Size is rounded to nearest power of 2 for performance reasons.
#[derive(Debug)]
pub struct HistoryBuffer<T: Default> {
    buffer: Vec<T>,
    base_index: usize,
}
impl<T: Default> HistoryBuffer<T> {
    pub fn new(size: usize) -> Self {
        let size = size.next_power_of_two();

        let mut buffer = Vec::with_capacity(size);
        buffer.resize_with(size, || T::default());

        Self {
            buffer,
            base_index: 0,
        }
    }

    pub fn push(&mut self, value: T) {
        self.base_index = self.wrap(self.base_index + 1);
        self.buffer[self.base_index] = value;
    }

    // Len is power of two, so we can use bitwise AND to wrap the index instead of modulo for
    // performance reasons.
    fn wrap(&self, index: usize) -> usize {index & (self.buffer.len()-1)}
}

impl<T: Default> Index<usize> for HistoryBuffer<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.buffer[self.wrap(self.base_index + self.buffer.len() - index)]
    }
}
