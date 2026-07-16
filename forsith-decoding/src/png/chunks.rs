use std::{any::Any, fmt::Display, io::{BufRead, Read}};
use crate::{CursorVec, DecodingError::{self, InvalidChunk}, Num, PngDecoder, png::{ColorType, PngReader}};
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

    fn read<R: BufRead>(reader: &mut PngReader<R>, len: usize) -> Result<Self, DecodingError>
    where Self: Sized;

    fn update_decoder<'a, R: BufRead, const D: u8, const F: u8>(self, decoder: &mut PngDecoder<'a, R, D, F>) -> Result<(), DecodingError>
    where Self: Sized;
}
pub fn downcast_chunkdata<T: ChunkData + Any>(b: Box<dyn ChunkData>) -> Result<Box<T>, Box<dyn Any>> {
    let b: Box<dyn Any> = b;
    b.downcast::<T>()
}

#[derive(Debug)]
pub struct IHDR {
    pub width: u32,
    pub _height: u32,
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

    fn read<R: BufRead>(reader: &mut PngReader<R>, len: usize) -> Result<Self, DecodingError>
    where Self: Sized {
        if len != 13 {return Err(InvalidChunk(ChunkType::Idat))}

        Ok(Self {
            width: u32::read_be(reader)?,
            _height: u32::read_be(reader)?,
            bit_depth: u8::read_be(reader)?,
            color_type: ColorType::try_from(u8::read_be(reader)?).map_err(|_| DecodingError::InvalidChunk(ChunkType::Ihdr))?,
            compression_method: u8::read_be(reader)?,
            filter_method:  u8::read_be(reader)?,
            interlace_method: u8::read_be(reader)?,
        })
    }

    fn update_decoder<'a, R: BufRead, const D: u8, const F: u8>(self, _decoder: &mut PngDecoder<'a, R, D, F>) -> Result<(), DecodingError>
    where Self: Sized {unreachable!()} // ihdr needs to have been read before the decoder is created, so this should never be called
}

// Will be read as IDAT chunk data
pub struct ZlibHeader {
    pub compression_method: u8,
    pub compression_info: u8,
    pub check: bool, // (CMF * 256 + FLG) % 31 == 0
    pub dict: bool,
    pub _flevel: u8,
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

    fn read<R: BufRead>(reader: &mut PngReader<R>, _len: usize) -> Result<Self, DecodingError>
    where Self: Sized {
        let cmf = reader.read_idat::<u8>()?;
        let flg = reader.read_idat::<u8>()?;

        Ok(Self {
            compression_method: cmf & 0b00001111,
            compression_info: (cmf & 0b11110000) >> 4,
            check: (cmf as u16 * 256 + flg as u16).is_multiple_of(31),
            dict: flg & 0b00100000 == 0b00100000,
            _flevel: (flg & 0b11000000) >> 6
        })
    }

    fn update_decoder<'a, R: BufRead, const D: u8, const F: u8>(self, decoder: &mut PngDecoder<'a, R, D, F>) -> Result<(), DecodingError>
    where Self: Sized {
        let lz77_buffer_size: usize = 1 << (self.compression_info + 8);

        decoder.deflate_buffer = CursorVec::new(lz77_buffer_size + lz77_buffer_size.next_multiple_of(decoder.scanline_bytes()));

        decoder.scanline_multiples = (decoder.deflate_buffer.capacity()-lz77_buffer_size) / decoder.scanline_bytes();

        Ok(())
    }
}

#[derive(Debug)]
pub struct ColorPalette {
    palette: [u32; 256],
    len: u8
}

impl ChunkData for ColorPalette {
    fn chunk_type(&self) -> ChunkType {ChunkType::Plte}

    fn validate(&self) -> Result<(), DecodingError> {Ok(())}

    fn read<R: BufRead>(reader: &mut PngReader<R>, len: usize) -> Result<Self, DecodingError>
    where Self: Sized {
        if len == 0 || len > 256*3 || !len.is_multiple_of(3) {return Err(InvalidChunk(ChunkType::Plte))}

        let mut palette = ColorPalette {palette: [0u32; 256], len: (len / 3) as u8};

        let mut rgb0 = [0u8; 4];
        for i in 0..palette.len as usize {
            reader.read_exact(&mut rgb0[..3])?;

            palette.palette[i] = u32::from_le_bytes(rgb0);
        }

        Ok(palette)
    }

    fn update_decoder<'a, R: BufRead, const D: u8, const F: u8>(self, decoder: &mut PngDecoder<'a, R, D, F>) -> Result<(), DecodingError>
    where Self: Sized {
        decoder.pallete = Some(self); Ok(())
    }
}
