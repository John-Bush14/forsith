use std::{io::BufRead};

use crate::{CursorVec, DecodingError, DestinationBuffer, PngDecoder};

use super::simd::filtering::SIMD_WIDTH;

impl<R: BufRead, const D: u8, const F: u8> PngDecoder<'_, R, D, F> {
    pub fn scanline_bytes(&self) -> usize {self.filterer.scanline_bytes()}
    pub fn scanline_pixel_bytes(&self) -> usize {self.filterer.scanline_pixel_bytes()}
}

pub fn scanline_bytes(width: u32, bitspp: u8) -> usize {
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
        dest.push_slice(self.prev_buffer());

        self.prev_buffer_mut().clear();

        Ok(())
    }

    #[inline]
    fn filter_and_push_scanline<const FILTER: u8, const STRIDE: usize>(&mut self, scanline: &[u8]) -> Result<(), DecodingError> {
        let mut alignment_bytes = scanline.len() % SIMD_WIDTH;

        if alignment_bytes < STRIDE && matches!(FILTER, 1 | 3 | 4) {alignment_bytes += SIMD_WIDTH;}
        if matches!(STRIDE, 1 | 2 | 3 | 6) && matches!(FILTER, 1 | 3 | 4) {alignment_bytes = scanline.len();}

        for (i, &b) in scanline.iter().enumerate().take(alignment_bytes) {
            let filtered_byte = self.filter::<FILTER>(b, i)?;
            self.cur_buffer_mut().push(filtered_byte);
        };

        for i in (alignment_bytes..scanline.len()).step_by(SIMD_WIDTH) {
            let filtered_bytes = self.filter_simd::<FILTER>(scanline, i)?;

            filtered_bytes.copy_to_slice(self.cur_buffer_mut().mut_slice(i..i + SIMD_WIDTH));
            self.cur_buffer_mut().advance(SIMD_WIDTH);
        }

        Ok(())
    }

    #[inline]
    fn filter<const FILTER: u8>(&self, b: u8, i: usize) -> Result<u8, DecodingError> {
        Ok(match FILTER {
            1 => b.wrapping_add(self.left_pixel(i)),
            2 => b.wrapping_add(self.upper_pixel(i)),
            3 => b.wrapping_add(((self.left_pixel(i) as u16 + self.upper_pixel(i) as u16) / 2) as u8),
            4 => b.wrapping_add(paeth_predictor(self.left_pixel(i), self.upper_pixel(i), self.left_upper_pixel(i))),
            _ => return Err(DecodingError::InvalidFilter(FILTER)),
        })
    }

    pub fn remaining_bytes(&self) -> usize {
        self.capacity() - self.scanline_buffers[0].len() - self.scanline_buffers[1].len()
    }

    pub fn capacity(&self) -> usize {self.scanline_buffers[0].capacity() * 2}

    pub fn scanline_bytes(&self) -> usize {self.scanline_buffers[0].capacity() + 1}
    pub fn scanline_pixel_bytes(&self) -> usize {self.scanline_buffers[0].capacity()}

    pub fn cur_buffer(&self) -> &[u8] {self.scanline_buffers[self.cur_buffer].as_slice()}
    pub fn prev_buffer(&self) -> &[u8] {self.scanline_buffers[1 - self.cur_buffer].as_slice()}

    pub fn cur_buffer_mut(&mut self) -> &mut CursorVec<u8> {&mut self.scanline_buffers[self.cur_buffer]}
    pub fn prev_buffer_mut(&mut self) -> &mut CursorVec<u8> {&mut self.scanline_buffers[1 - self.cur_buffer]}

    pub fn switch_buffers(&mut self) {self.cur_buffer = 1 - self.cur_buffer;}

    pub fn left_pixel(&self, i: usize) -> u8 {
        if i < self.stride {
            return 0;
        }

        unsafe {*self.cur_buffer().get_unchecked(i - self.stride)}
    }

    pub fn upper_pixel(&self, i: usize) -> u8 {
        if self.prev_buffer().is_empty() {
            return 0;
        }

        unsafe {*self.prev_buffer().get_unchecked(i)}
    }

    pub fn left_upper_pixel(&self, i: usize) -> u8 {
        if i < self.stride || self.prev_buffer().is_empty() {
            return 0;
        }

        unsafe {*self.prev_buffer().get_unchecked(i - self.stride)}
    }
}

#[inline]
fn paeth_predictor(a: u8, b: u8, c: u8) -> u8 {
    let (a, b, c) = (a as i32, b as i32, c as i32);

    let pa = (b - c).abs();
    let pb = (a - c).abs();
    let pc = (a + b - 2 * c).abs();

    let b_lt_a = ((pb - pa) as u32 >> 31) as i8; // wraps if negative
    let c_lt_a = ((pc - pa) as u32 >> 31) as i8;
    let c_lt_b = ((pc - pb) as u32 >> 31) as i8; // -1 makes it strict inequality

    let a_mask = ((b_lt_a^1) & (c_lt_a^1)).wrapping_neg() as u8;
    let b_mask = (b_lt_a & (c_lt_b^1)).wrapping_neg() as u8;
    let c_mask = !(a_mask | b_mask);

    (a_mask & a as u8) | (b_mask & b as u8) | (c_mask & c as u8)
}
