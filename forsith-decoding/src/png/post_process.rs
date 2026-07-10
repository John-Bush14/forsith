use std::io::BufRead;

use crate::{DecodingError, PngDecoder};

impl<R: BufRead, const D: u8, const F: u8> PngDecoder<'_, R, D, F> {
    pub fn consume_inflated_scanline(&mut self, dest: &mut [u8]) -> Result<(), DecodingError> {
        self.push_previous_scanline(dest)?;

        self.switch_scanline_buffers();

        let filter = self.deflate_buffer_mut().pop_last().unwrap();

        match filter {
            0 => {self.filter_and_push_scanline::<0>()?},
            1 => {self.filter_and_push_scanline::<1>()?},
            2 => {self.filter_and_push_scanline::<2>()?},
            3 => {self.filter_and_push_scanline::<3>()?},
            4 => {self.filter_and_push_scanline::<4>()?},
            _ => return Err(DecodingError::InvalidFilter(filter)),
        }

        Ok(())
    }

    fn filter_and_push_scanline<const FILTER: u8>(&mut self) -> Result<(), DecodingError> {
        let scanline_pixel_bytes = self.scanline_bytes() - 1;

        for i in 0..scanline_pixel_bytes {
            let raw_byte = self.deflate_buffer_mut().pop_last().unwrap();
            let filtered_byte = self.filter::<FILTER>(raw_byte, i)?;
            self.cur_scanline_buffer_mut().push(filtered_byte);
        };

        Ok(())
    }

    pub fn switch_scanline_buffers(&mut self) {self.cur_scanline_buffer = 1 - self.cur_scanline_buffer;}

    #[inline]
    fn filter<const FILTER: u8>(&mut self, b: u8, i: usize) -> Result<u8, DecodingError> {
        Ok(match FILTER {
            0 => b,
            1 => b.wrapping_add(self.left_pixel(i)),
            2 => b.wrapping_add(self.upper_pixel(i)),
            4 => b.wrapping_add(paeth_predictor(self.left_pixel(i), self.upper_pixel(i), self.left_upper_pixel(i))),
            3 => b.wrapping_add(((self.left_pixel(i) as u16 + self.upper_pixel(i) as u16) / 2) as u8),
            _ => return Err(DecodingError::InvalidFilter(F)),
        })
    }

    pub fn left_pixel(&self, i: usize) -> u8 {
        if i < self.source_bytespp as usize {
            return 0;
        }

        self.cur_scanline_buffer()[i - self.source_bytespp as usize]
    }

    pub fn cur_scanline_buffer(&self) -> &Vec<u8> {&self.scanline_buffers[self.cur_scanline_buffer]}
    pub fn prev_scanline_buffer(&self) -> &Vec<u8> {&self.scanline_buffers[1 - self.cur_scanline_buffer]}

    pub fn cur_scanline_buffer_mut(&mut self) -> &mut Vec<u8> {&mut self.scanline_buffers[self.cur_scanline_buffer]}
    pub fn prev_scanline_buffer_mut(&mut self) -> &mut Vec<u8> {&mut self.scanline_buffers[1 - self.cur_scanline_buffer]}

    pub fn upper_pixel(&self, i: usize) -> u8 {
        if self.prev_scanline_buffer().is_empty() {
            return 0;
        }

        self.prev_scanline_buffer()[i]
    }

    pub fn left_upper_pixel(&self, i: usize) -> u8 {
        if i < self.source_bytespp as usize || self.prev_scanline_buffer().is_empty() {
            return 0;
        }

        self.prev_scanline_buffer()[i - self.source_bytespp as usize]
    }

    pub fn remaining_scanline_buffers_bytes(&self) -> usize {
        self.scanline_bytes()*2 - self.scanline_buffers[0].len() - self.scanline_buffers[1].len()
    }

    pub fn scanline_bytes(&self) -> usize {self.scanline_buffers[0].capacity() + 1}
}

pub fn scanline_bytes(width: u32, bitspp: u8) -> usize {
    (width as usize * bitspp as usize) / 8 + 1
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
