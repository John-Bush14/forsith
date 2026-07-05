use std::io::{self, Read};
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
    EarlyClose(ChunkType, u32)
}
impl From<DecodingError> for io::Error {
    fn from(err: DecodingError) -> Self {
        io::Error::new(io::ErrorKind::Other, err)
    }
}

fn read_exact_array<const N: usize, R: Read>(reader: &mut R) -> io::Result<[u8; N]> {
    let mut buf = [0u8; N];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}
pub trait Num {
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
    const MAX: Self = u32::MAX;
}
impl Num for u8 {
    fn read_be<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(u8::from_be_bytes(read_exact_array::<1, _>(reader)?))
    }
    fn read_le<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(u8::from_le_bytes(read_exact_array::<1, _>(reader)?))
    }
    const BIT_DEPTH: u8 = 8;
    const MAX: Self = u8::MAX;
}
