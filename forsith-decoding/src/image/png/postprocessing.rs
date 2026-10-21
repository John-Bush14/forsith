use crate::image::{
    ChannelLayout, PixelFormat, PngDecoder, SampleType,
    outputconverting::OutputWriter,
    png::{
        ColorType,
        chunks::{Ihdr},
    },
};

pub const MAX_STRIDE: usize = 8;

impl PngDecoder<'_> {
    #[inline(always)]
    #[must_use]
    pub const fn scanline_bytes(&self) -> usize {
        self.pass.scanline_bytes()
    }
    #[inline(always)]
    #[must_use]
    pub const fn scanline_pixel_bytes(&self) -> usize {
        self.pass.scanline_pixel_bytes()
    }
    #[must_use]
    pub const fn max_scanline_bytes(&self) -> usize {
        self.pass.max_scanline_bytes()
    }
}

#[derive(Debug)]
pub struct PostProcessor {
    stride: usize,
    stored_format: PixelFormat,
}

impl PostProcessor {
    pub const fn new(ihdr: &Ihdr) -> Self {
        let stored_format = PixelFormat::new(
            SampleType::from_bit_depth_unsigned(ihdr.channel_depth).unwrap(),
            match ihdr.color_type {
                ColorType::Truecolor => ChannelLayout::Truecolor,
                ColorType::GrayscaleAlpha => ChannelLayout::GrayscaleAlpha,
                ColorType::TruecolorAlpha => ChannelLayout::TruecolorAlpha,
                ColorType::Grayscale | ColorType::Indexed => ChannelLayout::Grayscale,
            },
        );

        let stored_bitspp = stored_format.bpp();
        let stride = (stored_bitspp as usize).div_ceil(8);

        Self {
            stride,
            stored_format,
        }
    }

    pub const fn stored_bpp(&self) -> usize {
        self.stored_format.bpp() as usize
    }
    pub const fn stored_format(&self) -> PixelFormat {
        self.stored_format
    }

    pub const fn stride(&self) -> usize {
        self.stride
    }
}
