use std::{any::Any, fmt::Display, io::BufRead};
use crate::{DecodingError, Num, PngDecoder, png::{ChunkReader, ColorType}};
use num_enum::{TryFromPrimitive, IntoPrimitive};

#[repr(u32)]
#[derive(TryFromPrimitive, IntoPrimitive, Clone, Copy, Debug, PartialEq)]
pub enum ChunkType {
    Ihdr = 0x49484452,
    Plte = 0x504C5445,
    Idat = 0x49444154,
    Iend = 0x49454E44,
    UnkownAncillerary
}
impl Display for ChunkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(u32::from(*self).to_be_bytes().iter().map(|b| *b as char).collect::<String>().as_str())
    }
}
impl ChunkType {
    pub fn is_critical(&self) -> bool {
        *self as u32 & 0x20000000 == 0
    }
}
pub fn is_chunk_type_critical(chunk_type_buffer: &[u8; 4]) -> bool {
    chunk_type_buffer[0] & 0x20 == 0
}



pub trait ChunkData: Any {
    fn chunk_type(&self) -> ChunkType; // &self needed for Box

    fn validate(&self) -> Result<(), DecodingError>;

    fn read<R: BufRead>(reader: &mut ChunkReader<R>) -> Result<Self, DecodingError>
    where Self: Sized;

    fn update_decoder<'a, R: BufRead, C: Num, const F: u8>(self, decoder: &mut PngDecoder<'a, R, C, F>) -> Result<(), DecodingError>
    where Self: Sized;
}
pub fn downcast_chunkdata<T: ChunkData + Any>(b: Box<dyn ChunkData>) -> Result<Box<T>, Box<dyn Any>> {
    let b: Box<dyn Any> = b;
    b.downcast::<T>()
}

#[derive(Debug)]
pub struct IHDR {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub color_type: ColorType,
    pub compression_method: u8,
    pub filter_method: u8,
    pub interlace_method: u8,
}

impl ChunkData for IHDR {
    fn chunk_type(&self) -> ChunkType {ChunkType::Ihdr}

    fn validate(&self) -> Result<(), DecodingError> {
        if !matches!((self.compression_method, self.filter_method, self.interlace_method), (0, 0, 0 | 1)) {
            return Err(DecodingError::InvalidChunk(self.chunk_type()));
        }

        if self.interlace_method == 1 {
            return Err(DecodingError::InteralacingNotSupported);
        }

        match (&self.bit_depth, &self.color_type) {
            (1 | 2 | 4 | 8 | 16, ColorType::Grayscale)
            | (8 | 16, ColorType::Truecolor)
            | (1 | 2 | 4 | 8, ColorType::Indexed)
            | (8 | 16, ColorType::GrayscaleAlpha)
            | (8 | 16, ColorType::TruecolorAlpha) => Ok(()),
            _ => Err(DecodingError::InvalidChunk(self.chunk_type())),
        }
    }

    fn read<R: BufRead>(reader: &mut ChunkReader<R>) -> Result<Self, DecodingError>
    where Self: Sized {
        Ok(Self {
            width: u32::read_be(reader)?,
            height: u32::read_be(reader)?,
            bit_depth: u8::read_be(reader)?,
            color_type: ColorType::try_from(u8::read_be(reader)?).map_err(|_| DecodingError::InvalidChunk(ChunkType::Ihdr))?,
            compression_method: u8::read_be(reader)?,
            filter_method:  u8::read_be(reader)?,
            interlace_method: u8::read_be(reader)?,
        })
    }

    fn update_decoder<'a, R: BufRead, C: Num, const F: u8>(self, _decoder: &mut PngDecoder<'a, R, C, F>) -> Result<(), DecodingError>
    where Self: Sized {unreachable!()} // ihdr needs to have been read before the decoder is created, so this should never be called
}

// Will be read as IDAT chunk data
pub struct ZlibHeader {
    pub compression_method: u8,
    pub compression_info: u8,
    pub check: bool, // (CMF * 256 + FLG) % 31 == 0
    pub dict: bool,
    pub flevel: u8,
}

impl ChunkData for ZlibHeader {
    fn chunk_type(&self) -> ChunkType {ChunkType::Idat}

    fn validate(&self) -> Result<(), DecodingError> {
        if self.compression_method != 8
            || self.compression_info > 7
            || self.dict
            || !self.check
        {
            return Err(DecodingError::InvalidChunk(ChunkType::Idat));
        }

        Ok(())
    }

    fn read<R: BufRead>(reader: &mut ChunkReader<R>) -> Result<Self, DecodingError>
    where Self: Sized {
        let cmf = u8::read_be(reader)?;
        let flg = u8::read_be(reader)?;

        Ok(Self {
            compression_method: cmf & 0b00001111,
            compression_info: (cmf & 0b11110000) >> 4,
            check: (cmf as u16 * 256 + flg as u16) % 31 == 0,
            dict: flg & 0b00100000 == 0b00100000,
            flevel: (flg & 0b11000000) >> 6
        })
    }

    fn update_decoder<'a, R: BufRead, C: Num, const F: u8>(self, decoder: &mut PngDecoder<'a, R, C, F>) -> Result<(), DecodingError>
    where Self: Sized {
        decoder.lz77_buffer_size = 1 << (self.compression_info + 8);
        Ok(())
    }
}
