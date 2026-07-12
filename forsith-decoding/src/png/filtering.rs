use std::{io::BufRead};

use crate::{CursorVec, DecodingError, DestinationBuffer, PngDecoder, png::simd::filtering::should_use_simd};

use super::simd::filtering::SIMD_WIDTH;

impl<R: BufRead, const D: u8, const F: u8> PngDecoder<'_, R, D, F> {
    pub fn scanline_bytes(&self) -> usize {self.filterer.scanline_bytes()}
    pub fn scanline_pixel_bytes(&self) -> usize {self.filterer.scanline_pixel_bytes()}
}

pub fn calculate_scanline_bytes(width: u32, bitspp: u8) -> usize {
    (width as usize * bitspp as usize) / 8 + 1
}

#[derive(Debug)]
pub struct Filterer {
    scanline_buffers: [CursorVec<u8>; 2],
    cur_buffer: usize,
    pub stride: usize,
}

impl Filterer {
    pub fn new(scanline_bytes: usize, stride: usize) -> Self {
        Self {
            scanline_buffers: [CursorVec::new(scanline_bytes-1), CursorVec::new(scanline_bytes-1)],
            cur_buffer: 0,
            stride
        }
    }

    pub fn consume_inflated_scanline<const D: u8, const F: u8>(&mut self, scanline: &[u8], dest: &mut DestinationBuffer<'_, D, F>) -> Result<(), DecodingError> {
        match self.stride {
            1 => self.consume_inflated_scanline_const_stride::<D, F, 1>(scanline, dest),
            2 => self.consume_inflated_scanline_const_stride::<D, F, 2>(scanline, dest),
            3 => self.consume_inflated_scanline_const_stride::<D, F, 3>(scanline, dest),
            4 => self.consume_inflated_scanline_const_stride::<D, F, 4>(scanline, dest),
            6 => self.consume_inflated_scanline_const_stride::<D, F, 6>(scanline, dest),
            _ => Err(DecodingError::InvalidStride(self.stride)),
        }
    }
    fn consume_inflated_scanline_const_stride<const D: u8, const F: u8, const STRIDE: usize>(&mut self, scanline: &[u8], dest: &mut DestinationBuffer<'_, D, F>) -> Result<(), DecodingError> {
        self.drain_previous_scanline(dest)?;

        self.switch_buffers();

        let filter = scanline[0];

        let scanline = &scanline[1..self.scanline_bytes()];

        if filter == 0 {
            self.cur_buffer_mut().push_slice(scanline); return Ok(());
        }

        match filter {
            1 => {self.filter_and_push_scanline::<1, STRIDE>(scanline)?},
            2 => {self.filter_and_push_scanline::<2, STRIDE>(scanline)?},
            3 => {self.filter_and_push_scanline::<3, STRIDE>(scanline)?},
            4 => {self.filter_and_push_scanline::<4, STRIDE>(scanline)?},
            _ => return Err(DecodingError::InvalidFilter(filter)),
        }

        Ok(())
    }

    pub fn drain_previous_scanline<const D: u8, const F: u8>(&mut self, dest: &mut DestinationBuffer<'_, D, F>) -> Result<(), DecodingError> {
        dest.push_slice(self.prev_buffer().as_slice());

        self.prev_buffer_mut().clear();

        Ok(())
    }

    #[inline]
    fn filter_and_push_scanline<const FILTER: u8, const STRIDE: usize>(&mut self, scanline: &[u8]) -> Result<(), DecodingError> {
        if !should_use_simd::<STRIDE, FILTER>() {
            for (i, &b) in scanline.iter().enumerate() {
                let filtered_byte = self.filter::<FILTER>(b, i)?;
                self.cur_buffer_mut().push(filtered_byte);
            };

            return Ok(());
        }

        let mut alignment_bytes = scanline.len() % SIMD_WIDTH;

        if alignment_bytes < STRIDE && matches!(FILTER, 1 | 3 | 4) {alignment_bytes += SIMD_WIDTH;}

        for (i, &b) in scanline.iter().enumerate().take(alignment_bytes) {
            let filtered_byte = self.filter::<FILTER>(b, i)?;
            self.cur_buffer_mut().push(filtered_byte);
        };

        for i in (alignment_bytes..scanline.len()).step_by(SIMD_WIDTH) {
            let filtered_bytes = self.filter_simd::<FILTER, STRIDE>(scanline, i)?;

            filtered_bytes.copy_to_slice(self.cur_buffer_mut().mut_slice(i..i + SIMD_WIDTH));
            self.cur_buffer_mut().advance(SIMD_WIDTH);
        }

        Ok(())
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

    pub fn capacity(&self) -> usize {self.scanline_buffers[0].capacity() * 2}

    pub fn scanline_bytes(&self) -> usize {self.scanline_buffers[0].capacity() + 1}
    pub fn scanline_pixel_bytes(&self) -> usize {self.scanline_buffers[0].capacity()}

    pub fn cur_buffer(&self) -> &CursorVec<u8> {&self.scanline_buffers[self.cur_buffer]}
    pub fn prev_buffer(&self) -> &CursorVec<u8> {&self.scanline_buffers[1 - self.cur_buffer]}

    pub fn cur_buffer_mut(&mut self) -> &mut CursorVec<u8> {&mut self.scanline_buffers[self.cur_buffer]}
    pub fn prev_buffer_mut(&mut self) -> &mut CursorVec<u8> {&mut self.scanline_buffers[1 - self.cur_buffer]}

    pub fn switch_buffers(&mut self) {self.cur_buffer = 1 - self.cur_buffer;}

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
