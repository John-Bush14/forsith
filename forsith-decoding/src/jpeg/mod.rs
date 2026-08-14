use std::io::Read;
use crate::{Channel, DecodingError, ImageDecoder, PixelFormat, parsing::{SegmentParser, SegmentHeader}};

mod parser;
use parser::JpegParser;

const JPEG_HEADER: [u8; 2] = [0xFF, 0xD8];

#[derive(Debug)]
pub struct JpegDecoder<'a, C: Channel, const F: u8> {
    phantom: std::marker::PhantomData<&'a C>,
}

impl<'a, C: Channel, const F: u8> ImageDecoder<'a, C, F> for JpegDecoder<'a, C, F> {
    fn open_validated<R: Read>(mut reader: R) -> Result<Self, DecodingError> where Self: Sized {
        check_header(&mut reader)?;

        let mut parser = JpegParser::new(reader);

        let _ = parser.parse_first_chunk()?;

        let mut decoder = Self {
            phantom: std::marker::PhantomData,
        };

        parser.parse_chunks(|h, b, d| {
            println!("{d:?}");

            b.consume(h.length() as _); Ok(())
        })?;

        Ok(Self {
            phantom: std::marker::PhantomData,
        })
    }

    fn read(&mut self, _buf: &mut [<C as Channel>::StorageType]) -> Result<usize, DecodingError> {
        todo!()
    }

    fn image_dimensions(&self) -> (usize, usize) {
        todo!()
    }

    fn min_buf_size(&self) -> usize {
        todo!()
    }

    fn source_bit_depth(&self) -> u8 {
        todo!()
    }

    fn source_pixel_format(&self) -> PixelFormat {
        todo!()
    }
}

fn check_header<R: Read>(reader: &mut R) -> Result<(), DecodingError> {
    let mut header = [0u8; 2];
    reader.read_exact(&mut header)?;
    if header != JPEG_HEADER {
        return Err(DecodingError::InccorectHeader(header.to_vec()))
    }
    Ok(())
}
