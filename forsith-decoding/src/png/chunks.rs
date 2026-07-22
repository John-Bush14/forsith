use std::{any::Any, fmt::Display, io::{BufRead, Read}, ops::{Index, IndexMut}, ptr::read};
use crate::{Channel, CursorVec, DecodingError::{self, InvalidChunk}, Int, PngDecoder, png::{ColorType, PngReader}};
use num_enum::{TryFromPrimitive, IntoPrimitive};

#[repr(u32)]
#[allow(non_camel_case_types)]
#[derive(TryFromPrimitive, IntoPrimitive, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChunkType {
    Ihdr = 0x49484452,
    Plte = 0x504C5445,
    Idat = 0x49444154,
    Iend = 0x49454E44,
    tRNS = 0x74524E53,
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


#[derive(Debug)]
pub struct Ihdr {
    pub width: u32,
    pub height: u32,
    pub channel_depth: u8,
    pub color_type: ColorType,
    pub compression_method: u8,
    pub filter_method: u8,
    pub interlace_method: u8,
}

impl Ihdr {
    pub fn validate(&self) -> Result<(), DecodingError> {
        if !matches!((self.compression_method, self.filter_method, self.interlace_method), (0, 0, 0 | 1)) {
            return Err(DecodingError::InvalidChunk(ChunkType::Ihdr));
        }

        match (&self.channel_depth, &self.color_type) {
            (1 | 2 | 4 | 8 | 16, ColorType::Grayscale)
            | (8 | 16, ColorType::Truecolor)
            | (1 | 2 | 4 | 8, ColorType::Indexed)
            | (8 | 16, ColorType::GrayscaleAlpha)
            | (8 | 16, ColorType::TruecolorAlpha) => Ok(()),
            _ => Err(DecodingError::InvalidChunk(ChunkType::Ihdr)),
        }
    }

    pub fn read<R: BufRead>(reader: &mut PngReader<R>, len: usize) -> Result<Self, DecodingError>
    where Self: Sized {
        if len != 13 {return Err(InvalidChunk(ChunkType::Idat))}

        Ok(Self {
            width: u32::read_be(reader)?,
            height: u32::read_be(reader)?,
            channel_depth: u8::read_be(reader)?,
            color_type: ColorType::try_from(u8::read_be(reader)?).map_err(|_| DecodingError::InvalidChunk(ChunkType::Ihdr))?,
            compression_method: u8::read_be(reader)?,
            filter_method:  u8::read_be(reader)?,
            interlace_method: u8::read_be(reader)?,
        })
    }
}


pub trait ChunkData: Any {
    #[allow(unused)]
    fn chunk_type(&self) -> ChunkType; // &self needed for Box

    fn update_decoder<'a, R: BufRead, C: Channel, const F: u8>(decoder: &mut PngDecoder<'a, R, C, F>) -> Result<(), DecodingError>
    where Self: Sized;
}

// Will be read as IDAT chunk data
pub struct ZlibHeader {}
impl ChunkData for ZlibHeader {
    fn chunk_type(&self) -> ChunkType {ChunkType::Idat}

    fn update_decoder<'a, R: BufRead, C: Channel, const F: u8>(decoder: &mut PngDecoder<'a, R, C, F>) -> Result<(), DecodingError>
    where
        Self: Sized,
    {
        let reader = &mut decoder.reader;
        let cmf = u8::read_le(reader)?;
        let flg = u8::read_le(reader)?;

        let compression_method = cmf & 0b00001111;
        let compression_info = (cmf & 0b11110000) >> 4;
        let dict = flg & 0b00100000 == 0b00100000;

        if compression_method != 8
            || compression_info > 7
            || dict
            || !(cmf as u16 * 256 + flg as u16).is_multiple_of(31)
        {
            return Err(DecodingError::InvalidChunk(ChunkType::Idat));
        }

        let lz77_buffer_size: usize = 1 << (compression_info + 8);

        decoder.deflate_buffer = CursorVec::new(lz77_buffer_size + lz77_buffer_size.next_multiple_of(decoder.scanline_bytes()));

        decoder.scanline_multiples = (decoder.deflate_buffer.capacity()-lz77_buffer_size) / decoder.scanline_bytes();

        Ok(())
    }
}

#[derive(Debug)]
pub struct ColorPalette {
    palette: [u32; 256],
    len: u16,
}

impl IndexMut<usize> for ColorPalette {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {&mut self.palette[index]}
}

impl Index<usize> for ColorPalette {
    type Output = u32;

    fn index(&self, index: usize) -> &Self::Output {&self.palette[index]}
}

impl ColorPalette {
    fn set_pixel_alpha(&mut self, i: usize, a: u8) {
        let pixel = &mut self[i];

        *pixel = (*pixel & 0x00FF_FFFF) | ((a as u32) << 24);
    }
}

impl ChunkData for ColorPalette {
    fn chunk_type(&self) -> ChunkType {ChunkType::Plte}

    fn update_decoder<'a, R: BufRead, C: Channel, const F: u8>(decoder: &mut PngDecoder<'a, R, C, F>) -> Result<(), DecodingError>
    where

        Self: Sized
    {

        let reader = &mut decoder.reader; let len = reader.cur_chunk_len();

        if len == 0 || len > 256*3 || !len.is_multiple_of(3) {return Err(InvalidChunk(ChunkType::Plte))}

        let mut palette = ColorPalette {palette: [0u32; 256], len: (len / 3) as u16};

        // default alpha value should be 255
        let mut rgba = [255u8; 4];
        for i in 0..palette.len as usize {
            reader.read_exact(&mut rgba[..3])?;

            palette.palette[i] = u32::from_le_bytes(rgba);
        }

        decoder.postprocessor.set_palette(palette); Ok(())
    }
}

#[allow(non_camel_case_types)]
pub struct tRNS {}
impl ChunkData for tRNS {
    fn chunk_type(&self) -> ChunkType {ChunkType::tRNS}

    fn update_decoder<'a, R: BufRead, C: Channel, const F: u8>(decoder: &mut PngDecoder<'a, R, C, F>) -> Result<(), DecodingError>
    where
        Self: Sized,
    {
        let reader = &mut decoder.reader; let len = reader.cur_chunk_len();

        if decoder.postprocessor.color_type() != ColorType::Indexed {
            let mask = ((1 << decoder.postprocessor.channel_depth() as u32) - 1) as u16;

            let channel_max = (1 << decoder.postprocessor.channel_depth()) - 1;
            let mut read_val = || -> Result<i64, DecodingError> {
                let d = (u16::read_be(reader)? & mask) as i64;

                Ok(if decoder.postprocessor.channel_depth() < 8 {
                    d * 255 / channel_max
                } else {d})
            };

            let alpha_color = match decoder.postprocessor.color_type() {
                ColorType::Grayscale => {
                    let g = read_val()?;

                    (g, g, g)
                },
                ColorType::Truecolor => (read_val()?, read_val()?, read_val()?),
                _ => return Ok(())
            };

            decoder.postprocessor.set_alpha_color(alpha_color);

            return Ok(());
        }

        if decoder.postprocessor.palette().is_none() || len == 0 || len > 256 {return Err(InvalidChunk(ChunkType::tRNS))}

        let palette = decoder.postprocessor.palette_mut().unwrap();
        for i in 0..len {
            let a = u8::read_le(reader)?;

            palette.set_pixel_alpha(i, a);
        }

        Ok(())
    }
}
