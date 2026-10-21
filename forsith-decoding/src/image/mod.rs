use crate::DecodingError;
use std::io::Read;

#[cfg(feature = "png")]
pub(crate) mod png;
#[cfg(feature = "png")]
pub use png::PngDecoder;

#[cfg(feature = "jpeg")]
#[allow(dead_code, unused_variables, unused_mut)]
pub(crate) mod jpeg;
#[cfg(feature = "jpeg")]
pub use jpeg::JpegDecoder;

#[allow(dead_code)]
mod outputconverting;

pub mod pixel;
pub use pixel::{ChannelLayout, PixelFormat, SampleType};

pub trait ImageDecoder<'a>: Sized {
    fn open<R: Read>(data: R, preffered_format: PixelFormat) -> Result<Self, DecodingError>;

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, DecodingError>;

    fn image_dimensions(&self) -> (usize, usize);

    fn pixel_format(&self) -> PixelFormat;

    fn min_buf_size(&self) -> usize {
        self.max_buf_size()
    }
    fn max_buf_size(&self) -> usize {
        let dim = self.image_dimensions();
        (dim.0 * dim.1 * self.pixel_format().bpp() as usize).div_ceil(8)
    }
}
