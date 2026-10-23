use crate::image::{
    ChannelLayout, PixelFormat, PngDecoder, SampleType,
    outputconverting::OutputWriter,
    png::{
        ColorType,
        chunks::{ColorPalette, Ihdr},
    },
};

pub struct PngOutputWriter {
    src_format: PixelFormat,
    dst_format: PixelFormat,
    alpha_color: Option<(i64, i64, i64)>,
    palette: Option<ColorPalette>,
}

impl PngDecoder<'_> {
    #[must_use]
    pub const fn output_format(&self) -> PixelFormat {
        self.output.dst_format()
    }

    #[must_use]
    pub const fn stored_format(&self) -> PixelFormat {
        self.output.src_format()
    }
}

const fn stored_format(ihdr: &Ihdr) -> PixelFormat {
    PixelFormat::new(
        SampleType::from_bit_depth_unsigned(ihdr.channel_depth).unwrap(),
        match ihdr.color_type {
            ColorType::Truecolor => ChannelLayout::Truecolor,
            ColorType::GrayscaleAlpha => ChannelLayout::GrayscaleAlpha,
            ColorType::TruecolorAlpha => ChannelLayout::TruecolorAlpha,
            ColorType::Grayscale | ColorType::Indexed => ChannelLayout::Grayscale,
        },
    )
}

impl PngOutputWriter {
    pub const fn new(ihdr: &Ihdr) -> Self {
        let src_format = stored_format(ihdr);

        let dst_format = PixelFormat::new(
            src_format.sample,
            match ihdr.color_type {
                ColorType::Indexed => ChannelLayout::TruecolorAlpha,
                _ => src_format.layout,
            },
        );

        Self {
            dst_format,
            src_format,
            alpha_color: None,
            palette: None,
        }
    }

    pub fn can_push(&self, dest: &OutputWriter<'_>, bytes: usize) -> bool {
        dest.remaining() * 8 / usize::from(self.dst_format.bpp())
            * usize::from(self.src_format.bpp())
            >= bytes * 8
    }

    pub fn src_to_dst_bytes(&self, src_bytes: usize) -> usize {
        (src_bytes * 8 * usize::from(self.dst_format.bpp()) / usize::from(self.src_format.bpp()))
            .div_ceil(8)
    }

    pub const fn set_alpha_color(&mut self, c: (i64, i64, i64)) {
        self.alpha_color = Some(c);
    }

    pub const fn set_palette(&mut self, palette: ColorPalette) {
        self.palette = Some(palette);
    }

    pub const fn palette(&self) -> Option<&ColorPalette> {
        self.palette.as_ref()
    }

    pub const fn palette_mut(&mut self) -> Option<&mut ColorPalette> {
        self.palette.as_mut()
    }

    pub const fn src_format(&self) -> PixelFormat {
        self.src_format
    }

    pub const fn dst_format(&self) -> PixelFormat {
        self.dst_format
    }

    pub fn write_scanline(&self, scanline: &[u8], dest: &mut OutputWriter<'_>) {
        if let Some(palette) = &self.palette {
            dest.write_paletted(scanline, &**palette);
        } else if let Some(alpha_color) = self.alpha_color {
            dest.write_alpha_color(scanline, alpha_color);
        } else {
            dest.write_slice(scanline);
        }
    }
}
