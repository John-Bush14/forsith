use std::io;

#[cfg(feature = "jpg")]
use crate::image::jpeg::markers::MarkerType;
use crate::{checksums::CRC32, image::png::ChunkType};
#[cfg(not(feature = "jpg"))]
type MarkerType = ();
use forsith_proc::{Display, Error, IsVariant};

#[derive(Error, Debug, IsVariant, Display)]
pub enum DecodingError {
    #[display("Incorrect header ({_0:?})")]
    InccorectHeader(Vec<u8>),
    #[display("Unexpected IO error while reading data ({_0})")]
    IOError(#[from] std::io::Error),
    #[display(
        "Stored ({_0:?}) and calculated ({_1:?}) CRC did not match, indicating data corruption."
    )]
    CRCMismatch(CRC32, CRC32), // calculated, store
    #[display(
        "Stored ({_1:#010X}) and calculated ({_0:#010X}) Adler32 checksum did not match, indicating incorrect (de)compression."
    )]
    Adler32Mismatch(u32, u32), // calculated, store
    #[display("Provided dest buf of size less than min_buf_size ({_0})")]
    TinyDestBuf(usize),

    // JPEG specific
    #[display("No marker found in expected position")]
    NoMarker,
    #[display("Tried to access uninitialized or out of bounds AC huffman tree ({_0})")]
    TriedToAccesInvalidAcTree(usize),
    #[display("Tried to access uninitialized or out of bounds DC huffman tree ({_0})")]
    TriedToAccesInvalidDcTree(usize),
    #[display("Tried to access uninitialized or out of bounds quant table ({_0})")]
    TriedToAccesInvalidQuantTable(usize),
    #[display("No EOI marker found")]
    NoEOI,
    #[display("No frame found in jpeg")]
    NoFrame,
    #[display("Invalid marker length (<2)")]
    InvalidMarkerLen,
    #[display("Invalid marker code ({_0:#04X})")]
    InvalidMarkerCode(u8),
    #[display("Marker contained invalid data ({_0:?})")]
    InvalidMarker(MarkerType),
    #[display("Marker '{_0:?}' should have occurred before current marker '{_1:?}', but did not")]
    MarkerShouldHaveOccurred(MarkerType, MarkerType),

    // PNG specific
    #[display("Critical chunk '{_0}' contains invalid data")]
    InvalidChunk(ChunkType),
    #[display("Unknown critical chunk type '{_0:?}'")]
    UnkownChunk([u8; 4]),
    #[display("First chunk is not IHDR, instead ({_0:?})")]
    NoIHDR(ChunkType),
    #[display("No IDAT chunk found")]
    NoIDAT,
    #[display("Multiple '{_0}' chunks found")]
    MultipleChunks(ChunkType),
    #[display("Attempted to close chunk '{_0}' with incorrect amount of bytes ({_1}) remaining")]
    IncorrectClose(ChunkType, usize),
    #[display("Block length ({_0}) and its one's complement ({_1}) did not match")]
    BlockLengthMismatch(u16, u16),
    #[display("Code length ({_0}) is too large")]
    InvalidCodeLength(u8),
    #[display("Tried to register huffman symbol with value larger than {_0} bytes can hold.")]
    InvalidSymbol(usize),
    #[display("Undefined huffman code ({_0:#010X}) found in deflate stream.")]
    UndefinedHuffmanCode(u32),
    #[display("Reserved compression method found in deflate stream.")]
    ReservedCompressionMethod,
    #[display("Invalid filter ({_0}) written at start of scanline.")]
    InvalidFilter(u8),
    #[display("Invalid backreference with distance 0 found in deflate stream.")]
    ZeroDistance,
    #[display("Invalid bytes per pixel ({_0}) calculated for image.")]
    InvalidStride(usize),
    #[display("No Plte chunk found for an index color type image")]
    NoPallete,
    #[display("Provided png did not contain a Iend chunk")]
    NoIend,
}
impl From<DecodingError> for io::Error {
    fn from(err: DecodingError) -> Self {
        Self::other(err)
    }
}
