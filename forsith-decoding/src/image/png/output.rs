use crate::image::{ChannelLayout, PixelFormat, outputconverting::OutputWriter, png::{ColorType, chunks::ColorPalette}};

pub struct PngOutputWriter {
    format: PixelFormat,
    alpha_color: Option<(i64, i64, i64)>,
    palette: Option<ColorPalette>,
}

impl PngOutputWriter {
    pub const fn new(color_type: ColorType, stored_format: PixelFormat) -> Self {
        let format = PixelFormat::new(
            stored_format.sample,
            match color_type {
                ColorType::Indexed => ChannelLayout::TruecolorAlpha,
                _ => stored_format.layout,
            },
        );

        Self {
            format,
            alpha_color: None,
            palette: None,
        }
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

    pub const fn format(&self) -> PixelFormat {
        self.format
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
