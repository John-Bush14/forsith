use std::{io::BufRead};

use crate::{Channel, CursorVec, DecodingError, OutputWriter, PixelFormat, PngDecoder, has_alpha, outputconverting::{OutputConverter, get_out_writer_func}, png::{ColorType, chunks::{ColorPalette, Ihdr}, postprocessing, simd::filtering::should_use_simd}, unpack};

use super::simd::filtering::SIMD_WIDTH;

impl<R: BufRead, C: Channel, const F: u8> PngDecoder<'_, R, C, F> {
    pub fn scanline_bytes(&self) -> usize {self.postprocessor.scanline_bytes()}
    pub fn scanline_pixel_bytes(&self) -> usize {self.postprocessor.scanline_pixel_bytes()}
}

pub fn calculate_scanline_bytes(width: u32, bitspp: u8) -> (usize, u8) {
    let scanline_bits = width as usize * bitspp as usize;

    (scanline_bits.div_ceil(8) + 1, ((8 - (scanline_bits % 8)) % 8) as u8)
}

#[derive(Debug)]
pub struct PostProcessor<C: Channel, const F: u8> {
    scanline_buffers: [CursorVec<u8>; 2],
    cur_buffer: usize,
    pub stride: usize,
    palette: Option<ColorPalette>,
    color_type: ColorType,
    bitspp: u8,
    scanline_padding: u8, // in bits
    channel_depth: u8,
    out_writer: OutputConverter<C, F>,
    alpha_color: Option<(i64, i64, i64)>,
    consuming_pass: Pass,
    draining_pass: Pass
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
            scanline_buffers: [CursorVec::new(scanline_bytes-1), CursorVec::new(scanline_bytes-1)],
            cur_buffer: 0,
            stride,
            palette: None,
            color_type,
            bitspp,
            scanline_padding,
            channel_depth,
            out_writer,
            alpha_color: None,
            consuming_pass: Default::default(),
            draining_pass: Default::default()
        }
    }

    pub fn consume_inflated_scanline(&mut self, scanline: &[u8], dest: &mut OutputWriter<'_, C, F>) -> Result<(), DecodingError> {
        self.drain_previous_scanline(dest)?;

        let filter = scanline[0];

        let scanline = &scanline[1..self.scanline_bytes()];

        if filter == 0 {
            self.cur_buffer_mut().push_slice(scanline);

            self.scanline_consumed(dest)?;

            return Ok(());
        }

        match filter {
            1 => {self.filter_and_push_scanline::<1>(scanline)?},
            2 => {self.filter_and_push_scanline::<2>(scanline)?},
            3 => {self.filter_and_push_scanline::<3>(scanline)?},
            4 => {self.filter_and_push_scanline::<4>(scanline)?},
            _ => return Err(DecodingError::InvalidFilter(filter)),
        }

        self.scanline_consumed(dest)?;

        Ok(())
    }

    pub fn pixel_format(&self) -> PixelFormat {PixelFormat::from(self.color_type())}

    pub fn drain_previous_scanline(&mut self, dest: &mut OutputWriter<'_, C, F>) -> Result<(), DecodingError> {
        if self.prev_buffer().is_empty() {self.switch_buffers(); return Ok(())}

        if self.color_type != ColorType::Indexed {
            self.write_slice(self.prev_buffer().as_slice(), dest, self.scanline_padding);
        } else {
            self.drain_previous_scanline_indexed(dest)?
        }

        self.scanline_drained(dest);

        self.prev_buffer_mut().clear();

        self.switch_buffers();

        Ok(())
    }

    pub fn drain_previous_scanline_indexed(&mut self, dest: &mut OutputWriter<'_, C, F>) -> Result<(), DecodingError> {
        let palette = unsafe {self.palette.as_ref().unwrap_unchecked()};
        let index_bits = self.bitspp / 3;

        let push_index = |index: u8| {
            let pixel = palette[index as usize].to_le_bytes();

            if has_alpha(F) {
                self.write_slice(&pixel, dest, 0);
            } else {
                self.write_slice(&pixel[..3], dest, 0);
            }
        };

        match index_bits {
            8 => self.prev_buffer().as_slice().iter().cloned().for_each(push_index),
            _ => unpack(self.prev_buffer().as_slice(), index_bits, self.scanline_padding, push_index)
        }


        Ok(())
    }

    fn write_slice(&self, slice: &[u8], dest: &mut OutputWriter<'_, C, F>, padding: u8) {
        (self.out_writer)(slice, dest, padding, self.alpha_color);
    }

    #[inline]
    fn filter_and_push_scanline<const FILTER: u8>(&mut self, scanline: &[u8]) -> Result<(), DecodingError> {
        if !should_use_simd::<FILTER>(self.stride) {
            for (i, &b) in scanline.iter().enumerate() {
                let filtered_byte = self.filter::<FILTER>(b, i)?;
                self.cur_buffer_mut().push(filtered_byte);
            };

            return Ok(());
        }

        let mut alignment_bytes = scanline.len() % SIMD_WIDTH;

        if matches!(FILTER, 1 | 3 | 4) && alignment_bytes < self.stride {alignment_bytes += SIMD_WIDTH;}

        for (i, &b) in scanline.iter().enumerate().take(alignment_bytes) {
            let filtered_byte = self.filter::<FILTER>(b, i)?;
            self.cur_buffer_mut().push(filtered_byte);
        };

        match self.stride {
            1 => self.filter_and_push_scanline_simd::<FILTER, 1>(alignment_bytes, scanline),
            2 => self.filter_and_push_scanline_simd::<FILTER, 2>(alignment_bytes, scanline),
            3 => self.filter_and_push_scanline_simd::<FILTER, 3>(alignment_bytes, scanline),
            4 => self.filter_and_push_scanline_simd::<FILTER, 4>(alignment_bytes, scanline),
            6 => self.filter_and_push_scanline_simd::<FILTER, 6>(alignment_bytes, scanline),
            8 => self.filter_and_push_scanline_simd::<FILTER, 8>(alignment_bytes, scanline),
            _ => unreachable!()
        }?;

        Ok(())
    }

    fn filter_and_push_scanline_simd<const FILTER: u8, const STRIDE: usize>(&mut self, alignment_bytes: usize, scanline: &[u8]) -> Result<(), DecodingError> {
        for i in (alignment_bytes..scanline.len()).step_by(SIMD_WIDTH) {
            let filtered_bytes = self.filter_simd::<FILTER, STRIDE>(scanline, i)?;

            filtered_bytes.copy_to_slice(self.cur_buffer_mut().mut_slice(i..i + SIMD_WIDTH));
            self.cur_buffer_mut().advance(SIMD_WIDTH);
        } Ok(())
    }

    #[inline]
    fn filter<const FILTER: u8>(&self, b: u8, i: usize) -> Result<u8, DecodingError> {
        Ok(match FILTER {
            1 => b.wrapping_add(self.left_byte(i)),
            2 => b.wrapping_add(self.upper_byte(i)),
            3 => b.wrapping_add(((self.left_byte(i) as u16 + self.upper_byte(i) as u16) / 2) as u8),
            4 => b.wrapping_add(paeth_predictor(self.left_byte(i), self.upper_byte(i), self.left_upper_byte(i))),
            _ => return Err(DecodingError::InvalidFilter(FILTER)),
        })
    }

    pub fn remaining_bytes(&self) -> usize {
        self.capacity() - self.scanline_buffers[0].len() - self.scanline_buffers[1].len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {self.prev_buffer().is_empty() && self.cur_buffer().is_empty()}

    pub fn capacity(&self) -> usize {self.scanline_buffers[0].capacity() * 2}

    pub fn scanline_bytes(&self) -> usize {self.scanline_buffers[0].capacity() + 1}
    pub fn scanline_pixel_bytes(&self) -> usize {self.scanline_buffers[0].capacity()}

    pub fn cur_buffer(&self) -> &CursorVec<u8> {unsafe {self.scanline_buffers.get_unchecked(self.cur_buffer)}}
    pub fn prev_buffer(&self) -> &CursorVec<u8> {unsafe {self.scanline_buffers.get_unchecked(1usize.unchecked_sub(self.cur_buffer))}}

    pub fn cur_buffer_mut(&mut self) -> &mut CursorVec<u8> {unsafe {self.scanline_buffers.get_unchecked_mut(self.cur_buffer)}}
    pub fn prev_buffer_mut(&mut self) -> &mut CursorVec<u8> {unsafe {self.scanline_buffers.get_unchecked_mut(1usize.unchecked_sub(self.cur_buffer))}}

    pub fn switch_buffers(&mut self) {self.cur_buffer = 1 - self.cur_buffer;}

    pub fn color_type(&self) -> ColorType {self.color_type}

    pub fn left_byte(&self, i: usize) -> u8 {
        if i < self.stride {
            return 0;
        }

        self.cur_buffer()[i - self.stride]
    }

    pub fn upper_byte(&self, i: usize) -> u8 {
        if self.prev_buffer().is_empty() {
            return 0;
        }

        self.prev_buffer()[i]
    }

    pub fn left_upper_byte(&self, i: usize) -> u8 {
        if i < self.stride || self.prev_buffer().is_empty() {
            return 0;
        }

        self.prev_buffer()[i - self.stride]
    }

    pub fn set_palette(&mut self, palette: ColorPalette) {
        self.palette = Some(palette);
    }

    pub fn palette(&self) -> Option<&ColorPalette> {self.palette.as_ref()}
    pub fn palette_mut(&mut self) -> Option<&mut ColorPalette> {self.palette.as_mut()}

    pub fn channel_depth(&self) -> u8 {self.channel_depth}

    pub fn set_alpha_color(&mut self, c: (i64, i64, i64)) {self.alpha_color = Some(c);}

    pub fn setup_interlacing(&mut self, ihdr: &Ihdr) {
        self.consuming_pass = Pass::new(ihdr, self);
        self.draining_pass = self.consuming_pass.clone();
    }
    pub fn update_dest(&self, dest: &mut OutputWriter<'_, C, F>) {
        self.draining_pass.update_dest(dest)
    }

    pub fn scanline_consumed(&mut self, dest: &mut OutputWriter<'_, C, F>) -> Result<(), DecodingError> {
        self.scanline_passed::<false>(dest)
    }

    pub fn scanline_drained(&mut self, dest: &mut OutputWriter<'_, C, F>) {
        self.scanline_passed::<true>(dest).unwrap();
    }

    #[inline(always)]
    fn scanline_passed<const DRAIN: bool>(&mut self, dest: &mut OutputWriter<'_, C, F>) -> Result<(), DecodingError> {
        let pass: &mut Pass = if DRAIN {&mut self.draining_pass} else {&mut self.consuming_pass};

        if pass.end_scanline_skip == 0 {return Ok(())}

        pass.cur_scanline += pass.end_scanline_scanlines_passed as usize;

        if pass.cur == 6 && pass.cur_scanline + 1 > pass.dim.1 {return Ok(());}
        if pass.cur_scanline >= pass.dim.1 {
            pass.cur += 1;

            let (start, stride, passed_scanlines) = PASSES[pass.cur as usize];

            if start.0 >= pass.dim.0 || start.1 >= pass.dim.1 {return self.scanline_passed::<DRAIN>(dest);}

            let width = ((pass.dim.0 - start.0 - 1) as u32).div_euclid(stride as u32) + 1;
            let alignment = (pass.dim.0 + start.0) as isize - (start.0 as isize + width as isize * stride as isize);

            pass.cur_scanline = start.1;
            pass.end_scanline_scanlines_passed = passed_scanlines;
            pass.end_scanline_skip = (((passed_scanlines as usize - 1) * pass.dim.0) as isize + alignment) as usize;
            pass.stride = stride;

            if !DRAIN {
                self.drain_previous_scanline(dest)?;
                self.drain_previous_scanline(dest)?;

                let (new_scanline_bytes, padding) = calculate_scanline_bytes(width, if self.color_type != ColorType::Indexed {self.bitspp} else {self.channel_depth});
                self.scanline_padding = padding;

                self.scanline_buffers.iter_mut().for_each(|b| {b.buffer.clear(); b.clear(); b.buffer.resize(new_scanline_bytes - 1, 0u8)});
            } else {
                dest.reset(); dest.advance(start.0 + start.1 * pass.dim.0);

                dest.set_stride(pass.stride);
            }
        } else if DRAIN {
            dest.advance(pass.end_scanline_skip);
        }

        Ok(())
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

#[inline]
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

            postprocessor.scanline_buffers.iter_mut().for_each(|b| b.buffer.resize(new_scanline_bytes - 1, 0u8));

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
