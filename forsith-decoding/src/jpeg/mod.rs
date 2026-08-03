use std::io::Read;
use crate::{Channel, DecodingError, ImageDecoder, PixelFormat};

#[derive(Debug)]
pub struct JpegDecoder<'a, R: Read, C: Channel, const F: u8> {
    phantom: std::marker::PhantomData<&'a (R, C)>,
}

impl<'a, R: Read, C: Channel, const F: u8> ImageDecoder<'a, C, F> for JpegDecoder<'a, R, C, F> {
    fn open_validated<R2: Read>(_data: R2) -> Result<Self, DecodingError> where Self: Sized {
        todo!()
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

enum _JpegMarker {
    Soi,
    Sof(u8, usize),
    Dht(usize),
    Dqt(usize),
    Dri,
    Sos(usize),
    Rst(u8),
    App(u8, usize),
    Com(usize),
    Eoi,
}
