use std::io;

use crate::{checksums::CRC32, jpeg::markers::MarkerType, png::ChunkType};
use derive_more::IsVariant;
use thiserror::Error;


#[derive(Error, Debug, IsVariant)]
pub enum DecodingError {
    #[error("Incorrect header ({0:?})")]
    InccorectHeader(Vec<u8>),
    #[error("Unexpected IO error while reading data ({0})")]
    IOError(#[from] std::io::Error),
    #[error("Stored ({0:?}) and calculated ({1:?}) CRC did not match, indicating data corruption.")]
    CRCMismatch(CRC32, CRC32), // calculated, store
    #[error("Stored ({1:#010X}) and calculated ({0:#010X}) Adler32 checksum did not match, indicating incorrect (de)compression.")]
    Adler32Mismatch(u32, u32), // calculated, store
    #[error("Provided dest buf of size less than min_buf_size ({0})")]
    TinyDestBuf(usize),

    // JPEG specific
    #[error("No marker found in expected position")]
    NoMarker,
    #[error("Tried to access uninitialized or out of bounds AC huffman tree ({0})")]
    TriedToAccesInvalidAcTree(usize),
    #[error("Tried to access uninitialized or out of bounds DC huffman tree ({0})")]
    TriedToAccesInvalidDcTree(usize),
    #[error("Tried to access uninitialized or out of bounds quant table ({0})")]
    TriedToAccesInvalidQuantTable(usize),
    #[error("No EOI marker found")]
    NoEOI,
    #[error("No frame found in jpeg")]
    NoFrame,
    #[error("Invalid marker length (<2)")]
    InvalidMarkerLen,
    #[error("Invalid marker code ({0:#04X})")]
    InvalidMarkerCode(u8),
    #[error("Marker contained invalid data ({0:?})")]
    InvalidMarker(MarkerType),
    #[error("Marker '{0:?}' should have occurred before current marker '{1:?}', but did not")]
    MarkerShouldHaveOccurred(MarkerType, MarkerType),

    // PNG specific
    #[error("Critical chunk '{0}' contains invalid data")]
    InvalidChunk(ChunkType),
    #[error("Unknown critical chunk type '{0:?}'")]
    UnkownChunk([u8; 4]),
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
    NoPallete,
    #[error("Provided png did not contain a Iend chunk")]
    NoIend
}
impl From<DecodingError> for io::Error {
    fn from(err: DecodingError) -> Self {
        Self::other(err)
    }
}
