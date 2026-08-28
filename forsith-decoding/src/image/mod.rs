use crate::{DecodingError, int::Int};
use std::fmt::Debug;
use derive_more::IsVariant;
use num_enum::{IntoPrimitive, TryFromPrimitive};
pub use png::PngDecoder;

pub(crate) mod png;
use std::io::Read;

pub(crate) mod jpeg;
pub use jpeg::JpegDecoder;

mod outputconverting;

#[repr(u8)]
#[derive(TryFromPrimitive, IntoPrimitive, IsVariant)]
pub enum PixelFormat {
    Native = 0,
    Grayscale = 1,
    Truecolor = 3,
    GrayscaleAlpha = 2,
    TruecolorAlpha = 4
}

const fn has_alpha(format: u8) -> bool {matches!(format, 2 | 4)}
const fn is_rgb(format: u8) -> bool {matches!(format, 3 | 4)}
const fn is_gray(format: u8) -> bool {matches!(format, 1 | 2)}

pub(crate) const fn bitspp<C: Channel, const F: u8>() -> u8 {C::BIT_DEPTH * F}
pub(crate) const fn bytespp<C: Channel, const F: u8>() -> u8 {C::BIT_DEPTH * F / 8}

pub trait ImageDecoder<'a, C: Channel, const F: u8> {
    fn open_validated<R: Read>(data: R) -> Result<Self, DecodingError> where Self: Sized;
    fn open<R: Read>(data: R) -> Result<Self, DecodingError> where Self: Sized {
        assert!((C::BIT_DEPTH * F).is_multiple_of(8));
        assert!(PixelFormat::try_from(F).is_ok());

        Self::open_validated(data)
    }

    fn read(&mut self, buf: &mut [C::StorageType]) -> Result<usize, DecodingError>;

    fn image_dimensions(&self) -> (usize, usize);

    fn min_buf_size(&self) -> usize;
    fn max_buf_size(&self) -> usize {
        let dim = self.image_dimensions();
        dim.0 * dim.1 * F as usize
    }

    fn source_bit_depth(&self) -> u8;
    fn source_pixel_format(&self) -> PixelFormat;
}

#[derive(Debug, PartialEq, Eq, IsVariant)]
pub enum ChannelType {
    Unsigned,
    Signed,
    // Float,
    // NormalizedFloat
}

pub trait Channel
where
    <Self::StorageType as TryFrom<i64>>::Error: Debug,
    <Self::StorageType as TryFrom<u64>>::Error: Debug
{
    type StorageType: Int;
    const BIT_DEPTH: u8;
    const MAX: u64;
    const MIN: i64;
    const TYPE: ChannelType;
}

impl<I: Int> Channel for I
where
    <I as TryFrom<i64>>::Error: Debug,
    <I as TryFrom<u64>>::Error: Debug
{
    type StorageType = I;
    const BIT_DEPTH: u8 = I::BIT_DEPTH;
    const MAX: u64 = I::MAX;
    const MIN: i64 = I::MIN;
    const TYPE: ChannelType = {
        if I::SIGNED { ChannelType::Signed } else { ChannelType::Unsigned }
    };
}


