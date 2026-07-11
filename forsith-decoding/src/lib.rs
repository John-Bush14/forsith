use std::{io::{self, Read}, ops::{BitAnd, BitOr, BitXor, Shl, Shr}};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use thiserror::Error;

mod tests;

mod png;
pub use png::PngDecoder;
use png::CRC32;


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
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {self.read(buf).map_err(|e| io::Error::new(io::ErrorKind::Other, e))}
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
    CRCMismatch(CRC32, CRC32), // calculated, store
    #[error("Stored ({1:#010X}) and calculated ({0:#010X}) Adler32 checksum did not match, indicating incorrect (de)compression.")]
    Adler32Mismatch(u32, u32), // calculated, store
    #[error("First chunk is not IHDR, instead ({0:?})")]
    NoIHDR(ChunkType),
    #[error("No IDAT chunk found")]
    NoIDAT,
    #[error("Multiple '{0}' chunks found")]
    MultipleChunks(ChunkType),
    #[error("Attempted to close chunk '{0}' with incorrect amount of bytes ({1}) remaining")]
    IncorrectClose(ChunkType, u32),
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
    ZeroDistance
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

        if self.bits_remaining > I::BIT_DEPTH {
            panic!("BitBuffer overflow: bits_remaining = {}, I::BIT_DEPTH = {}", self.bits_remaining, I::BIT_DEPTH);
        }
    }
}

pub struct DestinationBuffer<'a, const D: u8, const F: u8> {
    buffer: &'a mut [u8],
    index: usize,
}

impl<'a, const D: u8, const F: u8> DestinationBuffer<'a, D, F> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self {
            buffer,
            index: 0
        }
    }

    pub fn push_byte(&mut self, b: u8) {
        self.buffer[self.index] = b;
        self.index += 1;
    }

    pub fn len(&self) -> usize {self.index}
    pub fn capacity(&self) -> usize {self.buffer.len()}
}
