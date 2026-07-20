use std::{cmp, io};

use crate::png::{ChunkType, CRC32};
use thiserror::Error;


#[derive(Error, Debug)]
pub enum DecodingError {
    #[error("Incorrect header ({0:?})")]
    InccorectHeader(Vec<u8>),
    #[error("Unexpected IO error while reading data")]
    IOError(#[from] std::io::Error),

    // PNG specific
    #[error("Critical chunk '{0}' contains invalid data")]
    InvalidChunk(ChunkType),
    #[error("Interlacing is not supported")]
    InteralacingNotSupported,
    #[error("Unknown critical chunk type '{0:?}'")]
    UnkownChunk([u8; 4]),
    #[error("Stored ({0:?}) and calculated ({1:?}) CRC did not match, indicating data corruption.")]
    CRCMismatch(u32, u32), // calculated, store
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
    #[error("No Plte chunk found for an index color type image")]
    NoPallete
}
impl From<DecodingError> for io::Error {
    fn from(err: DecodingError) -> Self {
        io::Error::other(err)
    }
}
