use std::{io::BufRead, ptr};
use crate::{Channel, CursorVec, DecodingError, OutputWriter, PixelFormat, PngDecoder, has_alpha, outputconverting::{OutputConverter, get_out_writer_func}, png::{ColorType, chunks::{ColorPalette, Ihdr}, simd::filtering::should_use_simd}, unpack};
use super::simd::filtering::SIMD_WIDTH;

pub const MAX_STRIDE: usize = 8;

impl<R: BufRead, C: Channel, const F: u8> PngDecoder<'_, R, C, F> {
    #[inline(always)]
    #[must_use]
    pub fn scanline_bytes(&self) -> usize {self.postprocessor.scanline_bytes()}
    #[inline(always)]
    #[must_use]
    pub fn scanline_pixel_bytes(&self) -> usize {self.postprocessor.scanline_pixel_bytes()}
    pub fn max_scanline_bytes(&self) -> usize {self.postprocessor.max_scanline_bytes()}
}

pub fn calculate_scanline_bytes(width: u32, bitspp: u8) -> (usize, u8) {
    let scanline_bits = width as usize * bitspp as usize;

    (scanline_bits.div_ceil(8) + 1, ((8 - (scanline_bits % 8)) % 8) as u8)
}

#[derive(Debug)]
pub struct PostProcessor<C: Channel, const F: u8> {
    scanline_bytes: usize,
    max_scanline_bytes: usize,
    pub stride: usize,
    palette: Option<ColorPalette>,
    color_type: ColorType,
    bitspp: u8,
    scanline_padding: u8, // in bits
    channel_depth: u8,
    out_writer: OutputConverter<C, F>,
    alpha_color: Option<(i64, i64, i64)>,
    adam7_pass: Pass,
}

impl<C: Channel, const F: u8> PostProcessor<C, F> {
    pub fn new(width: u32, color_type: ColorType, channel_depth: u8) -> Self
    {
        let bitspp = PixelFormat::from(color_type) as u8 * channel_depth;

        let (scanline_bytes, scanline_padding) = calculate_scanline_bytes(width, if color_type != ColorType::Indexed {bitspp} else {channel_depth});

        let stride = (bitspp as usize).div_ceil(8);

        let out_format = into_outconverter_pixel_format::<F>(color_type) as u8;
        let out_writer = get_out_writer_func::<C, F>(if color_type != ColorType::Indexed {channel_depth} else {8}, out_format, false);

        Self {
            scanline_bytes,
            max_scanline_bytes: scanline_bytes,
            stride,
            palette: None,
            color_type,
            bitspp,
            scanline_padding,
            channel_depth,
            out_writer,
            alpha_color: None,
            adam7_pass: Default::default(),
        }
    }

    pub fn filter_inflated_scanline(&mut self, mut scanline: *mut u8, dest: &mut OutputWriter<'_, C, F>) -> Result<(), DecodingError> {
        let filter = unsafe {*scanline};

        unsafe {scanline = scanline.add(1)};

        let filtered_scanline = unsafe {std::slice::from_raw_parts(scanline, self.scanline_pixel_bytes())};

        if filter == 0 {
            self.emit_filtered_scanline(filtered_scanline, dest);

            self.scanline_consumed(scanline, dest);

            return Ok(());
        }

        match filter {
            1 => {self.filter_scanline_inplace::<1>(scanline)},
            2 => {self.filter_scanline_inplace::<2>(scanline)},
            3 => {self.filter_scanline_inplace::<3>(scanline)},
            4 => {self.filter_scanline_inplace::<4>(scanline)},
            _ => return Err(DecodingError::InvalidFilter(filter)),
        }

        self.emit_filtered_scanline(filtered_scanline, dest);

        self.scanline_consumed(scanline, dest);

        Ok(())
    }

    pub fn emit_filtered_scanline(&mut self, scanline: &[u8], dest: &mut OutputWriter<'_, C, F>) {
        if self.color_type != ColorType::Indexed {
            self.write_slice(scanline, dest, self.scanline_padding);
        } else {
            self.emit_indexed_scanline(scanline, dest)
        }
    }

    pub fn emit_indexed_scanline(&mut self, scanline: &[u8], dest: &mut OutputWriter<'_, C, F>) {
        let palette = unsafe {self.palette.as_ref().unwrap_unchecked()};
        let index_bits = self.bitspp / 3;

        let mut push_index = |index: u8| {
            let pixel = palette[index as usize].to_le_bytes();

            if has_alpha(F) {
                self.write_slice(&pixel, dest, 0);
            } else {
                self.write_slice(&pixel[..3], dest, 0);
            }
        };

        match index_bits {
            8 => scanline.iter().cloned().for_each(push_index),
            _ => unpack::<false>(scanline, index_bits, self.scanline_padding, |bs| for &b in bs {push_index(b)})
        }
    }

    #[inline(always)]
    fn write_slice(&self, slice: &[u8], dest: &mut OutputWriter<'_, C, F>, padding: u8) {
        (self.out_writer)(slice, dest, padding, self.alpha_color);
    }

    fn filter_scanline_inplace<const FILTER: u8>(&mut self, scanline: *mut u8) {
        let mut cur = scanline;
        let mut up = unsafe {scanline.sub(self.scanline_bytes())};
        unsafe {
            #[cfg(debug_assertions)]
            assert!(!up.sub(self.stride).is_null() && !cur.add(self.scanline_bytes()).is_null(), "vec scanline ptr points too is not big enough");
        }

        if FILTER == 4 {
            unsafe {up.sub(self.stride).write_bytes(0, self.stride);}
        }

        let scanline_bytes = if FILTER != 2 {
            (cur, up) = self.filter_inplace_scalar::<FILTER, true>(self.stride.min(self.scanline_pixel_bytes()), cur, up);
            self.scanline_pixel_bytes().saturating_sub(self.stride)
        } else {
            self.scanline_pixel_bytes()
        };

        if !should_use_simd::<FILTER>(self.stride) {
            let _= self.filter_inplace_scalar::<FILTER, false>(scanline_bytes, cur, up);

            return;
        }

        let alignment_bytes = scanline_bytes % SIMD_WIDTH;

        (cur, up) = self.filter_inplace_scalar::<FILTER, false>(alignment_bytes, cur, up);

        let simd_iterations = (scanline_bytes - alignment_bytes) / SIMD_WIDTH;
        match self.stride {
            1 => self.filter_inplace_simd::<FILTER, 1>(simd_iterations, cur, up),
            2 => self.filter_inplace_simd::<FILTER, 2>(simd_iterations, cur, up),
            3 => self.filter_inplace_simd::<FILTER, 3>(simd_iterations, cur, up),
            4 => self.filter_inplace_simd::<FILTER, 4>(simd_iterations, cur, up),
            6 => self.filter_inplace_simd::<FILTER, 6>(simd_iterations, cur, up),
            8 => self.filter_inplace_simd::<FILTER, 8>(simd_iterations, cur, up),
            _ => unreachable!()
        };
    }

    #[inline]
    fn filter_inplace_scalar<const FILTER: u8, const BORDER: bool>(&mut self, n: usize, mut cur: *mut u8, mut up: *mut u8) -> (*mut u8, *mut u8) {
        for _ in 0..n {
            self.filter::<FILTER, BORDER>(cur, up);
            (cur, up) = unsafe {(cur.add(1), up.add(1))};
        };

        (cur, up)
    }

    fn filter_inplace_simd<const FILTER: u8, const STRIDE: usize>(&mut self, n: usize, mut cur: *mut u8, mut up: *mut u8) {
        for _ in 0..n {
            self.filter_simd::<FILTER, STRIDE>(cur, up);

            (cur, up) = unsafe {(cur.add(SIMD_WIDTH), up.add(SIMD_WIDTH))};
        }
    }

    #[inline(always)]
    fn filter<const FILTER: u8, const BORDER: bool>(&self, cur: *mut u8, up: *mut u8) {
        let r = match FILTER {
            1 => self.left_byte::<BORDER>(cur),
            2 => self.upper_byte(up),
            3 => ((self.left_byte::<BORDER>(cur) as u16 + self.upper_byte(up) as u16) / 2) as u8,
            4 => paeth_predictor(self.left_byte::<BORDER>(cur), self.upper_byte(up), self.left_upper_byte(up)),
            _ => unreachable!(),
        };

        unsafe {cur.write(r.wrapping_add(*cur))};
    }

    #[must_use]
    pub fn scanline_bytes(&self) -> usize {self.scanline_bytes}
    #[must_use]
    pub fn scanline_pixel_bytes(&self) -> usize {self.scanline_bytes() - 1}
    pub fn max_scanline_bytes(&self) -> usize {self.max_scanline_bytes}

    pub fn color_type(&self) -> ColorType {self.color_type}

    #[inline]
    pub fn left_byte<const BORDER: bool>(&self, cur: *mut u8) -> u8 {
        if !BORDER {unsafe {*cur.sub(self.stride)}} else {0}
    }

    #[inline]
    pub fn upper_byte(&self, up: *mut u8) -> u8 {
        unsafe {*up}
    }

    #[inline]
    pub fn left_upper_byte(&self, up: *mut u8) -> u8 {
        unsafe {*up.sub(self.stride)}
    }

    pub fn set_palette(&mut self, palette: ColorPalette) {
        self.palette = Some(palette);
    }

    pub fn palette(&self) -> Option<&ColorPalette> {self.palette.as_ref()}
    pub fn palette_mut(&mut self) -> Option<&mut ColorPalette> {self.palette.as_mut()}

    pub fn channel_depth(&self) -> u8 {self.channel_depth}

    pub fn set_alpha_color(&mut self, c: (i64, i64, i64)) {self.alpha_color = Some(c);}

    pub fn setup_interlacing(&mut self, ihdr: &Ihdr) {
        self.adam7_pass = Pass::new(ihdr, self);
    }
    pub fn update_dest(&self, dest: &mut OutputWriter<'_, C, F>) {
        self.adam7_pass.update_dest(dest)
    }

    pub fn scanline_consumed(&mut self, scanline: *mut u8, dest: &mut OutputWriter<'_, C, F>) {
        self.scanline_passed(scanline, dest)
    }

    #[inline(always)]
    fn scanline_passed(&mut self, scanline: *mut u8, dest: &mut OutputWriter<'_, C, F>) {
        let pass = &mut self.adam7_pass;

        if pass.end_scanline_skip == 0 {return;}

        pass.cur_scanline += pass.end_scanline_scanlines_passed as usize;

        if pass.cur == 6 && pass.cur_scanline + 1 > pass.dim.1 {return;}
        if pass.cur_scanline >= pass.dim.1 {
            pass.cur += 1;

            let (start, stride, passed_scanlines) = PASSES[pass.cur as usize];

            if start.0 >= pass.dim.0 || start.1 >= pass.dim.1 {return self.scanline_passed(scanline, dest);}

            let width = ((pass.dim.0 - start.0 - 1) as u32).div_euclid(stride as u32) + 1;
            let alignment = (pass.dim.0 + start.0) as isize - (start.0 as isize + width as isize * stride as isize);

            pass.cur_scanline = start.1;
            pass.end_scanline_scanlines_passed = passed_scanlines;
            pass.end_scanline_skip = (((passed_scanlines as usize - 1) * pass.dim.0) as isize + alignment) as usize;
            pass.stride = stride;

            let (new_scanline_bytes, padding) = calculate_scanline_bytes(width, if self.color_type != ColorType::Indexed {self.bitspp} else {self.channel_depth});
            self.scanline_padding = padding;

            dest.reset(); dest.advance(start.0 + start.1 * pass.dim.0);
            dest.set_stride(pass.stride);

            unsafe {
                scanline.add(self.scanline_pixel_bytes()).sub(new_scanline_bytes).write_bytes(0, new_scanline_bytes);
            }

            self.scanline_bytes = new_scanline_bytes;
        } else {
            dest.advance(pass.end_scanline_skip);
        }
    }
}

const PASSES: [((usize, usize), usize, u8); 7] = [
    ((0, 0), 8, 8),
    ((4, 0), 8, 8),
    ((0, 4), 4, 8),
    ((2, 0), 4, 4),
    ((0, 2), 2, 4),
    ((1, 0), 2, 2),
    ((0, 1), 1, 2)
];

pub fn into_outconverter_pixel_format<const F: u8>(color_type: ColorType) -> PixelFormat {
    if color_type == ColorType::Indexed && has_alpha(F) {
        PixelFormat::TruecolorAlpha
    } else {PixelFormat::from(color_type)}
}

fn paeth_predictor(a: u8, b: u8, c: u8) -> u8 {
    let (a, b, c) = (a as i16, b as i16, c as i16);

    let pa = (b - c).unsigned_abs();
    let pb = (a - c).unsigned_abs();
    let pc = (a + b - 2 * c).unsigned_abs();

    (if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }) as u8
}

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
    fn new<const F: u8>(ihdr: &Ihdr, postprocessor: &mut PostProcessor<impl Channel, F>) -> Pass {
        let (end_scanline_skip, stride) =  if ihdr.interlace_method == 1 {
            let width = (ihdr.width - 1).div_euclid(8) + 1;
            let alignment = ihdr.width as isize - width as isize * 8;

            let (new_scanline_bytes, padding) = calculate_scanline_bytes(width, ihdr.channel_depth * match ihdr.color_type {ColorType::Indexed => 1, c => PixelFormat::from(c) as u8});
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
            end_scanline_scanlines_passed: 8
        }
    }

    pub fn update_dest<const F: u8>(&self, dest: &mut OutputWriter<'_, impl Channel, F>) {
        dest.set_stride(self.stride);
    }
}
