use std::io::Read;
use crate::{DecodingError, ImageDecoder, Num, PixelFormat, png::{chunks::{IHDR, downcast_chunkdata}}};
use num_enum::TryFromPrimitive;

mod chunks;
pub use chunks::{ChunkType, ChunkData};

mod chunkreader;
pub use chunkreader::ChunkReader;

mod checksum;

const PNG_HEADER: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

#[repr(u8)]
#[derive(Debug, TryFromPrimitive, Clone, Copy)]
enum ColorType {
    Grayscale = 0,
    Truecolor = 2,
    Indexed = 3,
    GrayscaleAlpha = 4,
    TruecolorAlpha = 6,
}
#[allow(clippy::from_over_into)]
impl Into<PixelFormat> for ColorType {
    fn into(self) -> PixelFormat {
        match self {
            ColorType::Grayscale => PixelFormat::Grayscale,
            ColorType::Truecolor => PixelFormat::Truecolor,
            ColorType::Indexed => todo!(),
            ColorType::GrayscaleAlpha => PixelFormat::GrayscaleAlpha,
            ColorType::TruecolorAlpha => PixelFormat::TruecolorAlpha,
        }
    }
}

#[derive(Debug)]
pub struct PngDecoder<'a, R: Read, C: Num, const F: u8> {
    reader: ChunkReader<R>,
    buffer: Vec<u8>,
    phantom: std::marker::PhantomData<&'a C>,
    ihdr: IHDR,
}

impl<'a, R: Read, C: Num, const F: u8> ImageDecoder<'a, R, C, F> for PngDecoder<'a, R, C, F> {
    fn open(mut reader: R) -> Result<Self, DecodingError> {
        check_header(&mut reader)?;

        let mut reader = ChunkReader::new(reader);

        let ihdr = read_ihdr(&mut reader)?;

        let mut decoder = Self {
            reader,
            buffer: Vec::with_capacity(0),
            phantom: std::marker::PhantomData,
            ihdr,
        };

        loop  {
            decoder.reader.open_chunk()?;

            if decoder.reader.cur_type() == ChunkType::Iend {
                return Err(DecodingError::NoIDAT);
            }

            decoder.update_with_chunk()?;

            if decoder.reader.cur_type() == ChunkType::Idat {
                break;
            }
        }

        Ok(decoder)
    }

    fn next(&mut self) -> Option<Result<&'a [u8], DecodingError>> {
        todo!()
    }

    fn bit_depth(&self) -> u8 {self.ihdr.bit_depth}

    fn pixel_format(&self) -> crate::PixelFormat {self.ihdr.color_type.into()}
}

impl<'a, R: Read, C: Num, const F: u8> PngDecoder<'a, R, C, F> {
    fn update_with_chunk(&mut self) -> Result<(), DecodingError> {
        let chunk_data = self.reader.read_data()?;

        if let Err(err) = chunk_data.validate() {
            if self.reader.cur_type().is_critical() {
                return Err(err);
            }
            return Ok(());
        }

        match self.reader.cur_type() {
            ChunkType::UnkownAncillerary | ChunkType::Iend => unreachable!(),
            ChunkType::Ihdr => Err(DecodingError::MultipleChunks(ChunkType::Ihdr)),
            _ => todo!()
        }
    }
}

fn check_header<R: Read>(data: &mut R) -> Result<(), DecodingError> {
    let header = crate::read_exact_array::<8,_>(data)?;
    if header != PNG_HEADER {
        return Err(DecodingError::InccorectHeader(header.to_vec()))
    }
    Ok(())
}

fn read_ihdr<R: Read>(reader: &mut ChunkReader<R>) -> Result<IHDR, DecodingError> {
    reader.open_chunk()?;

    if reader.cur_type() != ChunkType::Ihdr {
        return Err(DecodingError::NoIHDR(reader.cur_type()));
    }

    let ihdr = *downcast_chunkdata::<IHDR>(reader.read_data()?).unwrap();
    ihdr.validate()?;

    Ok(ihdr)
}
