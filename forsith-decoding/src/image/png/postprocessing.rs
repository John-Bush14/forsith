pub use crate::SIMD_WIDTH;
use crate::{
    DecodingError,
    bit::unpack,
    image::{
        ChannelLayout, PixelFormat, PngDecoder, SampleType,
        outputconverting::OutputWriter,
        png::{
            ColorType,
            chunks::{ColorPalette, Ihdr},
        },
    },
};
use core::simd::prelude::*;

pub const MAX_STRIDE: usize = 8;

impl PngDecoder<'_> {
    #[inline(always)]
    #[must_use]
    pub const fn scanline_bytes(&self) -> usize {
        self.postprocessor.scanline_bytes()
    }
    #[inline(always)]
    #[must_use]
    pub const fn scanline_pixel_bytes(&self) -> usize {
        self.postprocessor.scanline_pixel_bytes()
    }
    #[must_use]
    pub const fn max_scanline_bytes(&self) -> usize {
        self.postprocessor.max_scanline_bytes()
    }
}

#[allow(clippy::cast_possible_truncation)]
pub const fn calculate_scanline_bytes(width: u32, bitspp: u8) -> (usize, u8) {
    let scanline_bits = width as usize * bitspp as usize;

    (
        scanline_bits.div_ceil(8) + 1,
        ((8 - (scanline_bits % 8)) % 8) as u8,
    )
}

#[derive(Debug)]
pub struct PostProcessor {
    scanline_bytes: usize,
    max_scanline_bytes: usize,
    stride: usize,
    palette: Option<ColorPalette>,
    color_type: ColorType,
    stored_format: PixelFormat,
    output_format: PixelFormat,
    scanline_padding: u8, // in bits
    channel_depth: u8,
    alpha_color: Option<(i64, i64, i64)>,
    adam7_pass: Pass,
}

impl PostProcessor {
    pub fn new(width: u32, color_type: ColorType, channel_depth: u8) -> Self {
        let stored_format = PixelFormat::new(
            SampleType::from_bit_depth_unsigned(channel_depth).unwrap(),
            match color_type {
                ColorType::Grayscale => ChannelLayout::Grayscale,
                ColorType::Truecolor => ChannelLayout::Truecolor,
                ColorType::GrayscaleAlpha => ChannelLayout::GrayscaleAlpha,
                ColorType::TruecolorAlpha => ChannelLayout::TruecolorAlpha,
                ColorType::Indexed => ChannelLayout::Grayscale,
            },
        );

        let output_format = PixelFormat::new(
            stored_format.sample,
            match color_type {
                ColorType::Indexed => ChannelLayout::TruecolorAlpha,
                _ => stored_format.layout,
            },
        );

        let stored_bitspp = stored_format.bpp();
        let (scanline_bytes, scanline_padding) = calculate_scanline_bytes(width, stored_bitspp);
        let stride = (stored_bitspp as usize).div_ceil(8);

        Self {
            scanline_bytes,
            max_scanline_bytes: scanline_bytes,
            stride,
            palette: None,
            color_type,
            scanline_padding,
            channel_depth,
            alpha_color: None,
            adam7_pass: Pass::default(),
            stored_format,
            output_format,
        }
    }

    pub fn emit_filtered_scanline(&self, scanline: &[u8], dest: &mut OutputWriter<'_>) {
        match self.color_type {
            ColorType::Indexed => self.emit_indexed_scanline(scanline, dest),
            _ => self.write_slice(scanline, dest, self.scanline_padding),
        }
    }

    pub fn emit_indexed_scanline(&self, scanline: &[u8], dest: &mut OutputWriter<'_>) {
        let palette = self.palette.as_ref().unwrap();
        let sample_ty = self.stored_format().sample;

        let mut push_index = |index: u8| {
            let pixel = palette[index as usize].to_le_bytes();

            self.write_slice(&pixel, dest, 0);
        };

        match sample_ty {
            SampleType::U8 => scanline.iter().copied().for_each(push_index),
            _ => unpack::<false>(
                scanline,
                sample_ty.bit_depth(),
                self.scanline_padding,
                |bs| {
                    for &b in bs {
                        push_index(b);
                    }
                },
            ),
        }
    }

    #[inline(always)]
    fn write_slice(&self, slice: &[u8], dest: &mut OutputWriter<'_>, _padding: u8) {
        if self.stored_format().sample.bit_depth() < 8 {
            todo!()
        }

        dest.write_slice(slice);
    }


    #[must_use]
    pub const fn scanline_bytes(&self) -> usize {
        self.scanline_bytes
    }
    #[must_use]
    pub const fn scanline_pixel_bytes(&self) -> usize {
        self.scanline_bytes() - 1
    }
    pub const fn max_scanline_bytes(&self) -> usize {
        self.max_scanline_bytes
    }

    pub const fn color_type(&self) -> ColorType {
        self.color_type
    }

    pub fn stored_bpp(&self) -> usize {
        self.stored_format.bpp() as usize
    }
    pub fn stored_format(&self) -> PixelFormat {
        self.stored_format
    }

    pub fn output_format(&self) -> PixelFormat {
        self.output_format
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

    pub const fn set_alpha_color(&mut self, c: (i64, i64, i64)) {
        self.alpha_color = Some(c);
    }

    pub fn setup_interlacing(&mut self, ihdr: &Ihdr) {
        self.adam7_pass = Pass::new(ihdr, self);
    }
    pub const fn update_dest(&self, dest: &mut OutputWriter<'_>) {
        self.adam7_pass.update_dest(dest);
    }

    #[inline(always)]
    pub fn scanline_passed(&mut self, scanlines: &mut [u8], i: usize, dest: &mut OutputWriter<'_>) {
        let pass = &mut self.adam7_pass;

        if pass.end_scanline_skip == 0 {
            return;
        }

        pass.cur_scanline += pass.end_scanline_scanlines_passed as usize;

        if pass.cur == 6 && pass.cur_scanline + 1 > pass.dim.1 {
            return;
        }
        if pass.cur_scanline >= pass.dim.1 {
            self.next_adam7_pass(scanlines, i, dest);
        } else {
            dest.advance(pass.end_scanline_skip);
        }
    }

    fn next_adam7_pass(&mut self, scanlines: &mut [u8], i: usize, dest: &mut OutputWriter<'_>) {
        let pass = &mut self.adam7_pass;
        pass.cur += 1;

        let (start, stride, passed_scanlines) = PASSES[pass.cur as usize];
        if start.0 >= pass.dim.0 || start.1 >= pass.dim.1 {
            return self.scanline_passed(scanlines, i, dest);
        }

        let width = pass.update(dest, start, stride, passed_scanlines);

        self.update_scanline_bytes(width, scanlines, i);
    }

    #[allow(clippy::cast_possible_truncation)]
    fn update_scanline_bytes(&mut self, width: u32, scanlines: &mut [u8], i: usize) {
        let (scanline_bytes, scanline_padding) =
            calculate_scanline_bytes(width, self.stored_bpp() as u8);

        let end = i + self.scanline_pixel_bytes();
        scanlines[end - self.stride - (scanline_bytes - 1)..end].fill(0);

        self.scanline_bytes = scanline_bytes;
        self.scanline_padding = scanline_padding;
    }

    pub fn stride(&self) -> usize {self.stride}
}

const PASSES: [((usize, usize), usize, u8); 7] = [
    ((0, 0), 8, 8),
    ((4, 0), 8, 8),
    ((0, 4), 4, 8),
    ((2, 0), 4, 4),
    ((0, 2), 2, 4),
    ((1, 0), 2, 2),
    ((0, 1), 1, 2),
];

#[derive(Debug, Default, Clone)]
pub struct Pass {
    cur: u8,
    stride: usize,
    dim: (usize, usize), // widht, height
    cur_scanline: usize,
    end_scanline_skip: usize,
    end_scanline_scanlines_passed: u8,
}

impl Pass {
    fn update(
        &mut self,
        dest: &mut OutputWriter<'_>,
        start: (usize, usize),
        stride: usize,
        passed_scanlines: u8,
    ) -> u32 {
        self.cur_scanline = start.1;
        self.end_scanline_scanlines_passed = passed_scanlines;
        self.stride = stride;
        self.update_dest(dest);
        dest.reset();
        dest.advance(start.0 + start.1 * self.dim.0);

        let width = (self.dim.0 - start.0 - 1).div_euclid(stride) + 1;

        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_possible_wrap
        )]
        {
            let alignment = (self.dim.0 + start.0) as isize
                - (start.0 as isize + width as isize * stride as isize);

            self.end_scanline_skip =
                (((passed_scanlines as usize - 1) * self.dim.0) as isize + alignment) as usize;
        }

        u32::try_from(width).unwrap()
    }

    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_possible_wrap
    )]
    fn new(ihdr: &Ihdr, postprocessor: &mut PostProcessor) -> Self {
        let (end_scanline_skip, stride) = if ihdr.interlace_method == 1 {
            let width = (ihdr.width - 1).div_euclid(8) + 1;
            let alignment = ihdr.width as isize - width as isize * 8;

            let (new_scanline_bytes, padding) =
                calculate_scanline_bytes(width, postprocessor.stored_bpp() as u8);
            postprocessor.scanline_padding = padding;

            postprocessor.scanline_bytes = new_scanline_bytes;

            (((ihdr.width as usize * 7) as isize + alignment) as usize, 8)
        } else {
            (0, 1)
        };

        Self {
            cur: 0,
            stride,
            dim: (ihdr.width as usize, ihdr.height as usize),
            cur_scanline: 0,
            end_scanline_skip,
            end_scanline_scanlines_passed: 8,
        }
    }

    pub const fn update_dest(&self, dest: &mut OutputWriter<'_>) {
        dest.set_stride(self.stride);
    }
}
