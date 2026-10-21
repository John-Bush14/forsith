use crate::DecodingError;
use forsith_proc::IsVariant;
use std::{fmt::Debug, io::Read};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PixelFormat {
    pub layout: ChannelLayout,
    pub sample: SampleType,
}

impl Default for PixelFormat {
    fn default() -> Self {
        Self {
            layout: ChannelLayout::TruecolorAlpha,
            sample: SampleType::U8,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, IsVariant)]
pub enum ChannelLayout {
    Grayscale,
    Truecolor,
    GrayscaleAlpha,
    TruecolorAlpha,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, IsVariant)]
pub enum SampleType {
    U1,
    U2,
    U4,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
}

impl SampleType {
    #[must_use]
    pub const fn bit_depth(self) -> u8 {
        match self {
            Self::U1 => 1,
            Self::U2 => 2,
            Self::U4 => 4,
            Self::U8 => 8,
            Self::U16 => 16,
            Self::U32 | Self::F32 => 32,
            Self::U64 | Self::F64 => 64,
        }
    }

    #[must_use]
    pub const fn from_bit_depth_unsigned(bit_depth: u8) -> Option<Self> {
        match bit_depth {
            1 => Some(Self::U1),
            2 => Some(Self::U2),
            4 => Some(Self::U4),
            8 => Some(Self::U8),
            16 => Some(Self::U16),
            32 => Some(Self::U32),
            64 => Some(Self::U64),
            _ => None,
        }
    }
}

impl ChannelLayout {
    #[must_use]
    pub const fn channel_count(self) -> u8 {
        match self {
            Self::Grayscale => 1,
            Self::Truecolor => 3,
            Self::GrayscaleAlpha => 2,
            Self::TruecolorAlpha => 4,
        }
    }

    #[must_use]
    pub const fn has_color(self) -> bool {
        matches!(self, Self::Truecolor | Self::TruecolorAlpha)
    }

    #[must_use]
    pub const fn has_alpha(self) -> bool {
        matches!(self, Self::GrayscaleAlpha | Self::TruecolorAlpha)
    }
}

impl PixelFormat {
    /// bits per pixel
    #[must_use]
    pub const fn bpp(self) -> u8 {
        self.layout.channel_count() * self.sample.bit_depth()
    }

    #[must_use]
    pub const fn new(sample: SampleType, layout: ChannelLayout) -> Self {
        Self { layout, sample }
    }
}

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
