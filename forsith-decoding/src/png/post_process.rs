use std::io::BufRead;

use crate::{DecodingError, DestinationBuffer, PngDecoder};

impl<R: BufRead, const D: u8, const F: u8> PngDecoder<'_, R, D, F> {

    pub fn scanline_bytes(&self) -> usize {self.filterer.scanline_bytes()}
    pub fn scanline_pixel_bytes(&self) -> usize {self.filterer.scanline_pixel_bytes()}
}

pub fn scanline_bytes(width: u32, bitspp: u8) -> usize {
    (width as usize * bitspp as usize) / 8 + 1
}

#[derive(Debug)]
pub struct Filterer {
    scanline_buffers: [Vec<u8>; 2],
    cur_buffer: usize,
    stride: usize
}

impl Filterer {
    pub fn new(scanline_bytes: usize, stride: usize) -> Self {
        Self {
            scanline_buffers: [Vec::with_capacity(scanline_bytes-1), Vec::with_capacity(scanline_bytes-1)],
            cur_buffer: 0,
            stride
        }
    }

    pub fn consume_inflated_scanline<const D: u8, const F: u8>(&mut self, scanline: &[u8], dest: &mut DestinationBuffer<'_, D, F>) -> Result<(), DecodingError> {
        self.drain_previous_scanline(dest)?;

        self.switch_buffers();

        let filter = scanline[0];

        match filter {
            0 => {self.filter_and_push_scanline::<0>(scanline)?},
            1 => {self.filter_and_push_scanline::<1>(scanline)?},
            2 => {self.filter_and_push_scanline::<2>(scanline)?},
            3 => {self.filter_and_push_scanline::<3>(scanline)?},
            4 => {self.filter_and_push_scanline::<4>(scanline)?},
            _ => return Err(DecodingError::InvalidFilter(filter)),
        }

        Ok(())
    }

    pub fn drain_previous_scanline<const D: u8, const F: u8>(&mut self, dest: &mut DestinationBuffer<'_, D, F>) -> Result<(), DecodingError> {
        for &b in self.prev_buffer() {
            dest.push_byte(b);
        }

        self.prev_buffer_mut().clear();

        Ok(())
    }

    #[inline]
    fn filter_and_push_scanline<const FILTER: u8>(&mut self, scanline: &[u8]) -> Result<(), DecodingError> {
        for (i, &b) in scanline[1..self.scanline_bytes()].iter().enumerate() {
            let filtered_byte = self.filter::<FILTER>(b, i)?;
            self.cur_buffer_mut().push(filtered_byte);
        };

        Ok(())
    }

    #[inline]
    fn filter<const FILTER: u8>(&self, b: u8, i: usize) -> Result<u8, DecodingError> {
        Ok(match FILTER {
            0 => b,
            1 => b.wrapping_add(self.left_pixel(i)),
            2 => b.wrapping_add(self.upper_pixel(i)),
            4 => b.wrapping_add(paeth_predictor(self.left_pixel(i), self.upper_pixel(i), self.left_upper_pixel(i))),
            3 => b.wrapping_add(((self.left_pixel(i) as u16 + self.upper_pixel(i) as u16) / 2) as u8),
            _ => return Err(DecodingError::InvalidFilter(FILTER)),
        })
    }

    pub fn remaining_bytes(&self) -> usize {
        self.capacity() - self.scanline_buffers[0].len() - self.scanline_buffers[1].len()
    }

    pub fn capacity(&self) -> usize {self.scanline_buffers[0].capacity() * 2}

    pub fn scanline_bytes(&self) -> usize {self.scanline_buffers[0].capacity() + 1}
    pub fn scanline_pixel_bytes(&self) -> usize {self.scanline_buffers[0].capacity()}

    pub fn cur_buffer(&self) -> &[u8] {&self.scanline_buffers[self.cur_buffer]}
    pub fn prev_buffer(&self) -> &[u8] {&self.scanline_buffers[1 - self.cur_buffer]}

    pub fn cur_buffer_mut(&mut self) -> &mut Vec<u8> {&mut self.scanline_buffers[self.cur_buffer]}
    pub fn prev_buffer_mut(&mut self) -> &mut Vec<u8> {&mut self.scanline_buffers[1 - self.cur_buffer]}

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
