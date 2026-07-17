use std::{io::BufRead};

use crate::{CursorVec, DecodingError, DestinationBuffer, PixelFormat, PngDecoder, has_alpha, png::{ColorType, chunks::ColorPalette, simd::filtering::should_use_simd}};

use super::simd::filtering::SIMD_WIDTH;

impl<R: BufRead, const D: u8, const F: u8> PngDecoder<'_, R, D, F> {
    pub fn scanline_bytes(&self) -> usize {self.postprocessor.scanline_bytes()}
    pub fn scanline_pixel_bytes(&self) -> usize {self.postprocessor.scanline_pixel_bytes()}
}

pub fn calculate_scanline_bytes(width: u32, bitspp: u8) -> (usize, u8) {
    let scanline_bits = width as usize * bitspp as usize;

    (scanline_bits.div_ceil(8) + 1, (scanline_bits % 8) as u8)
}

#[derive(Debug)]
pub struct PostProcessor<const F: u8> {
    scanline_buffers: [CursorVec<u8>; 2],
    cur_buffer: usize,
    pub stride: usize,
    palette: Option<ColorPalette>,
    color_type: ColorType,
    bitspp: u8,
    scanline_padding: u8, // in bits
    channel_depth: u8
}

impl<const F: u8> PostProcessor<F> {
    pub fn new(width: u32, color_type: ColorType, channel_depth: u8) -> Self {
        let bitspp = PixelFormat::from(color_type) as u8 * channel_depth;

        let (scanline_bytes, scanline_padding) = calculate_scanline_bytes(width, if color_type != ColorType::Indexed {bitspp} else {channel_depth});

        let stride = bitspp as usize / 8;

        Self {
            scanline_buffers: [CursorVec::new(scanline_bytes-1), CursorVec::new(scanline_bytes-1)],
            cur_buffer: 0,
            stride,
            palette: None,
            color_type,
            bitspp,
            scanline_padding,
            channel_depth
        }
    }

    pub fn consume_inflated_scanline<const D: u8>(&mut self, scanline: &[u8], dest: &mut DestinationBuffer<'_, D, F>) -> Result<(), DecodingError> {
        self.drain_previous_scanline(dest)?;

        self.switch_buffers();

        let filter = scanline[0];

        let scanline = &scanline[1..self.scanline_bytes()];

        if filter == 0 {
            self.cur_buffer_mut().push_slice(scanline); return Ok(());
        }

        match filter {
            1 => {self.filter_and_push_scanline::<1>(scanline)?},
            2 => {self.filter_and_push_scanline::<2>(scanline)?},
            3 => {self.filter_and_push_scanline::<3>(scanline)?},
            4 => {self.filter_and_push_scanline::<4>(scanline)?},
            _ => return Err(DecodingError::InvalidFilter(filter)),
        }

        Ok(())
    }

    pub fn pixel_format(&self) -> PixelFormat {PixelFormat::from(self.color_type())}

    pub fn drain_previous_scanline<const D: u8>(&mut self, dest: &mut DestinationBuffer<'_, D, F>) -> Result<(), DecodingError> {
        if self.color_type != ColorType::Indexed {
            dest.push_slice(self.prev_buffer().as_slice(), self.pixel_format() as u8, self.channel_depth, self.scanline_padding);
        } else {
            self.drain_previous_scanline_indexed(dest)?
        }

        self.prev_buffer_mut().clear();

        Ok(())
    }

    pub fn drain_previous_scanline_indexed<const D: u8>(&mut self, dest: &mut DestinationBuffer<'_, D, F>) -> Result<(), DecodingError> {
        let palette = unsafe {self.palette.as_ref().unwrap_unchecked()};
        let index_bits = self.bitspp / 3;

        for i in 0..self.prev_buffer().len() {
            let mut byte = self.prev_buffer()[i];

            let mut iterations = 8/index_bits;

            if i == self.prev_buffer().len() - 1 {iterations -= self.scanline_padding / index_bits}

            for _ in 0..iterations {
                let index = if index_bits == 8 {byte} else {byte >> (8 - index_bits)};

                let pixel = palette[index as usize].to_le_bytes();

                let pixel = if has_alpha(F) {&pixel} else {&pixel[..3]};

                if has_alpha(F) {
                    dest.push_8bit_pixel::<4>(pixel);
                } else {
                    dest.push_8bit_pixel::<3>(pixel);
                }

                if index_bits < 8 {byte <<= index_bits};
            }
        }

        Ok(())
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
