use crate::{
    DecodingError::{self, InvalidChunk}, buffers::CursorVec, image::{
        ChannelLayout, PngDecoder, png::{ColorType, parser::ChunkHeader, postprocessing::MAX_STRIDE},
    },
};
use forsith_proc::{Deref, Index, IsVariant};
use forsith_proc::{IntoNum, TryFromNum};
use std::{any::Any, fmt::Display, io::Read};

#[repr(u32)]
#[allow(non_camel_case_types)]
#[derive(TryFromNum, IntoNum, Clone, Copy, Debug, PartialEq, Eq, Default, IsVariant)]
pub enum ChunkType {
    Ihdr = 0x4948_4452,
    Plte = 0x504C_5445,
    Idat = 0x4944_4154,
    Iend = 0x4945_4E44,
    tRNS = 0x7452_4E53,
    #[default]
    UnkownAncillerary = 0,
}
impl Display for ChunkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            u32::from(*self)
                .to_be_bytes()
                .iter()
                .map(|b| *b as char)
                .collect::<String>()
                .as_str(),
        )
    }
}
impl ChunkType {
    pub const fn is_critical(self) -> bool {
        self as u32 & 0x2000_0000 == 0
    }
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
    pub const fn validate(&self) -> Result<(), DecodingError> {
        if !matches!(
            (
                self.compression_method,
                self.filter_method,
                self.interlace_method
            ),
            (0, 0, 0 | 1)
        ) {
            return Err(DecodingError::InvalidChunk(ChunkType::Ihdr));
        }

        match (&self.channel_depth, &self.color_type) {
            (1 | 2 | 4 | 8 | 16, ColorType::Grayscale)
            | (
                8 | 16,
                ColorType::Truecolor | ColorType::GrayscaleAlpha | ColorType::TruecolorAlpha,
            )
            | (1 | 2 | 4 | 8, ColorType::Indexed) => Ok(()),
            _ => Err(DecodingError::InvalidChunk(ChunkType::Ihdr)),
        }
    }

    pub fn read<R: Read>(reader: &mut R, len: usize) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        if len != 13 {
            return Err(InvalidChunk(ChunkType::Idat));
        }

        Ok(Self {
            width: reader.read_be::<u32>()?,
            height: reader.read_be::<u32>()?,
            channel_depth: reader.read_be::<u8>()?,
            color_type: ColorType::try_from(reader.read_be::<u8>()?)
                .map_err(|()| DecodingError::InvalidChunk(ChunkType::Ihdr))?,
            compression_method: reader.read_be::<u8>()?,
            filter_method: reader.read_be::<u8>()?,
            interlace_method: reader.read_be::<u8>()?,
        })
    }
}

pub trait ChunkData: Any {
    fn update_decoder(
        decoder: &mut PngDecoder<'_>,
        chunk_header: &ChunkHeader,
        data: &mut CursorVec<u8>,
    ) -> Result<(), DecodingError>
    where
        Self: Sized;
}

// Will be read as IDAT chunk data
pub struct ZlibDataStream {}
impl ChunkData for ZlibDataStream {
    fn update_decoder(
        decoder: &mut PngDecoder<'_>,
        chunk_header: &ChunkHeader,
        data: &mut CursorVec<u8>,
    ) -> Result<(), DecodingError>
    where
        Self: Sized,
    {
        data.get_mut().shrink_to(chunk_header.len());

        std::mem::swap(&mut *decoder.compressed_data, data);
        let reader = &mut decoder.compressed_data;

        let lz77_buffer_size = decoder.deflate.open_stream(&mut **reader)?;

        let scanline_padding = decoder.max_scanline_bytes() + MAX_STRIDE;
        decoder.deflate_buffer = CursorVec::new(
            scanline_padding
                + lz77_buffer_size
                + lz77_buffer_size.next_multiple_of(decoder.max_scanline_bytes()),
        );

        decoder.deflate_buffer.consume(scanline_padding);
        decoder.deflate_buffer_tail = scanline_padding;
        decoder.last_adler_update_i = scanline_padding;

        decoder.scanline_multiples =
            (decoder.deflate_buffer.capacity() - lz77_buffer_size - MAX_STRIDE)
                / decoder.max_scanline_bytes()
                - 1;

        Ok(())
    }
}

#[derive(Debug, Index, Deref)]
pub struct ColorPalette {
    #[index]
    #[deref]
    palette: [u32; 256],
    len: u16,
}

impl ColorPalette {
    const fn set_pixel_alpha(&mut self, i: usize, a: u8) {
        let pixel = &mut self.palette[i];

        *pixel = (*pixel & 0x00FF_FFFF) | ((a as u32) << 24);
    }
}

impl ChunkData for ColorPalette {
    fn update_decoder(
        decoder: &mut PngDecoder<'_>,
        chunk_header: &ChunkHeader,
        reader: &mut CursorVec<u8>,
    ) -> Result<(), DecodingError>
    where
        Self: Sized,
    {
        let len = chunk_header.len();

        if len == 0 || len > 256 * 3 || !len.is_multiple_of(3) {
            return Err(InvalidChunk(ChunkType::Plte));
        }

        let mut palette = Self {
            palette: [0u32; 256],
            len: u16::try_from(len / 3).unwrap(),
        };

        // default alpha value should be 255
        let mut rgba = [255u8; 4];
        for i in 0..palette.len as usize {
            reader.read_exact(&mut rgba[..3])?;

            palette.palette[i] = u32::from_le_bytes(rgba);
        }

        decoder.output.set_palette(palette);
        Ok(())
    }
}

#[allow(non_camel_case_types)]
pub struct tRNS {}
impl ChunkData for tRNS {
    #[allow(clippy::cast_possible_truncation)]
    fn update_decoder(
        decoder: &mut PngDecoder<'_>,
        chunk_header: &ChunkHeader,
        reader: &mut CursorVec<u8>,
    ) -> Result<(), DecodingError>
    where
        Self: Sized,
    {
        let len = chunk_header.len();

        let is_indexed = decoder.stored_format().is_grayscale() && decoder.output_format().has_color();
        if !is_indexed {
            let stored_depth = decoder.postprocessor.stored_format().sample.bit_depth();

            let mask = ((1u32 << u32::from(stored_depth)) - 1) as u16;

            let channel_max = (1 << stored_depth) - 1;
            let mut read_val = || -> Result<i64, DecodingError> {
                let d = i64::from(reader.read_be::<u16>()? & mask);

                Ok(if stored_depth < 8 {
                    d * 255 / channel_max
                } else {
                    d
                })
            };

            let alpha_color = match decoder.postprocessor.stored_format().layout {
                ChannelLayout::Grayscale => {
                    let g = read_val()?;

                    (g, g, g)
                }
                ChannelLayout::TruecolorAlpha => (read_val()?, read_val()?, read_val()?),
                _ => return Ok(()),
            };

            decoder.output.set_alpha_color(alpha_color);

            todo!();

            #[allow(unreachable_code)]
            return Ok(());
        }

        if decoder.output.palette().is_none() || len == 0 || len > 256 {
            return Err(InvalidChunk(ChunkType::tRNS));
        }

        let palette = decoder.output.palette_mut().unwrap();
        for i in 0..len {
            let a = reader.read_le::<u8>()?;

            palette.set_pixel_alpha(i, a);
        }

        Ok(())
    }
}
