use std::io::BufRead;

use crate::{DecodingError, PngDecoder, png::PngFilter};

impl<R: BufRead, const D: u8, const F: u8> PngDecoder<'_, R, D, F> {
    pub fn consume_inflated_scanline(&mut self, dest: &mut [u8]) -> Result<(), DecodingError> {
        let filter = self.deflate_buffer_mut().pop_last().unwrap();

        println!("Filter: {}", filter);

        match filter {
            0 => {self.filter_and_push_scanline::<0>(dest)?},
            1 => {self.filter_and_push_scanline::<1>(dest)?},
            2 => {self.filter_and_push_scanline::<2>(dest)?},
            3 => {self.filter_and_push_scanline::<3>(dest)?},
            4 => {self.filter_and_push_scanline::<4>(dest)?},
            _ => return Err(DecodingError::InvalidFilter(filter)),
        }

        Ok(())
    }

    fn filter_and_push_scanline<const FILTER: u8>(&mut self, dest: &mut [u8]) -> Result<(), DecodingError> {
        for i in 0..self.scanline_bytes()-1 {
            let raw_byte = self.deflate_buffer_mut().pop_last().unwrap();
            let filtered_byte = self.filter::<FILTER>(raw_byte, i)?;
            self.scanline_buffer.push(filtered_byte);

            if self.scanline_buffer.remaining_space() == 0 {
                let b = self.scanline_buffer.pop_last().unwrap();
                self.push_filtered_byte(b, dest)?;
            }
        }

        Ok(())
    }

    #[inline]
    fn filter<const FILTER: u8>(&mut self, b: u8, i: usize) -> Result<u8, DecodingError> {
        Ok(match FILTER {
            0 => b,
            1 => b.wrapping_add(self.left_pixel(i)),
            2 => b.wrapping_add(self.upper_pixel()),
            3 => b.wrapping_add(((self.left_pixel(i) as u16 + self.upper_pixel() as u16) / 2) as u8),
            4 => {
                let A = self.left_pixel(i) as i16;
                let B = self.upper_pixel() as i16;
                let C = self.left_upper_pixel(i) as i16;


                let p: i16 = A + B - C;

                let pa = (p - A).unsigned_abs();
                let pb = (p - B).unsigned_abs();
                let pc = (p - C).unsigned_abs();

                let minp = pa.min(pb).min(pc);

                let min = if minp == pa {
                    A
                } else if minp == pb {
                    B
                } else {
                    C
                };

                b.wrapping_add(min as u8)
            },
            _ => return Err(DecodingError::InvalidFilter(F)),
        })
    }

    pub fn left_pixel(&self, i: usize) -> u8 {
        if i < self.source_bytespp as usize {
            return 0;
        }

        self.scanline_buffer[self.source_bytespp as usize - 1]
    }

    pub fn upper_pixel(&self) -> u8 {
        self.scanline_buffer[self.scanline_bytes() - 2]
    }

    pub fn left_upper_pixel(&self, i: usize) -> u8 {
        if i < self.source_bytespp as usize {
            return 0;
        }

        self.scanline_buffer[self.source_bytespp as usize + self.scanline_bytes() - 2]
    }

    pub fn scanline_bytes(&self) -> usize {
        (self.ihdr.width as usize * self.source_bitspp as usize) / 8 + 1
    }
}
