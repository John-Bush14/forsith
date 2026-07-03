use std::{any::Any, io::Read, ops::Deref};
use crate::{AssetKind, Decoder, DecodingError, png::{chunks::{Chunk, IHDR, downcast_chunkdata}, crc::CRCReader}};

mod chunks;
pub use chunks::{ChunkType, ChunkData};
use num_enum::TryFromPrimitive;

mod crc;

const PNG_HEADER: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
const IHDR_HEADER: [u8; 4] = [0x49, 0x48, 0x44, 0x52];

#[repr(u8)]
#[derive(Debug, TryFromPrimitive)]
enum ColorType {
    Grayscale = 0,
    Truecolor = 2,
    Indexed = 3,
    GrayscaleAlpha = 4,
    TruecolorAlpha = 6,
}

#[derive(Debug)]
pub struct PngDecoder<'a, R: Read> {
    reader: CRCReader<R>,
    buffer: Vec<u8>,
    phantom: std::marker::PhantomData<&'a ()>,
    ihdr: IHDR,
    current_idat: Option<Chunk>,
}

impl<'a, R: Read> Decoder<R> for PngDecoder<'a, R> {
    type Chunk = &'a [u8];

    const KIND: AssetKind = AssetKind::Image;

    fn open(mut reader: R) -> Result<Self, DecodingError> {
        check_header(&mut reader)?;

        let mut reader = CRCReader::new(reader);

        let ihdr = read_ihdr(&mut reader)?;

        let mut decoder = Self {
            reader,
            buffer: Vec::new(),
            phantom: std::marker::PhantomData,
            ihdr,
            current_idat: None,
        };

        loop  {
            let mut chunk = Chunk::open(&mut decoder.reader)?;

            if chunk.r#type() == ChunkType::UnkownAncillerary {continue;}
            if chunk.r#type() == ChunkType::Idat {
                decoder.current_idat = Some(chunk);
                return Ok(decoder);
            }

            let chunk_data = chunk.read_data(&mut decoder.reader)?;

            if let Err(err) = chunk_data.validate() {
                if chunk.r#type().is_critical() {
                    return Err(err);
                }
                continue;
            }

            match chunk.r#type() {
                ChunkType::UnkownAncillerary | ChunkType::Idat => unreachable!(),
                ChunkType::Ihdr => return Err(DecodingError::MultipleChunks(chunk.r#type())),
                _ => todo!()
            };
        }

    }
}

impl<'a, R: Read> Iterator for PngDecoder<'a, R> {
    type Item = Result<<Self as Decoder<R>>::Chunk, DecodingError>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

fn check_header<R: Read>(data: &mut R) -> Result<(), DecodingError> {
    let header = crate::read_exact_array::<8,_>(data)?;
    if header != PNG_HEADER {
        return Err(DecodingError::InccorectHeader(header.to_vec()))
    }
    Ok(())
}

fn read_ihdr<R: Read>(reader: &mut CRCReader<R>) -> Result<IHDR, DecodingError> {
    let mut chunk = Chunk::open(reader)?;

    if chunk.r#type() != ChunkType::Ihdr {
        return Err(DecodingError::NoIHDR(chunk.r#type()));
    }

    let ihdr = *downcast_chunkdata::<IHDR>(chunk.read_data(reader)?).unwrap();
    ihdr.validate()?;

    Ok(ihdr)
}
