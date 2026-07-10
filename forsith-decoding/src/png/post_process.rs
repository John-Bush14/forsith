use std::io::BufRead;

use crate::{DecodingError, PngDecoder, png::PngFilter};

impl<R: BufRead, const D: u8, const F: u8> PngDecoder<'_, R, D, F> {
    pub fn filter(&mut self, b: u8) -> Result<Option<u8>, DecodingError> {
        let cur_filter = &self.cur_scanline.0;
        let rem_bytes = self.cur_scanline.1;

        if rem_bytes == 0 {
            let Ok(new_filter) = b.try_into() else {return Err(DecodingError::InvalidFilter(b));};
            self.cur_scanline = (new_filter, self.scanline_bytes() - 1);
            return Ok(None);
        }


        let corrected = match cur_filter {
            crate::png::PngFilter::None => b,
            crate::png::PngFilter::Sub => b.wrapping_add(self.left_pixel()),
            crate::png::PngFilter::Up => b.wrapping_add(self.upper_pixel()),
            crate::png::PngFilter::Average => b.wrapping_add(((self.left_pixel() as u16 + self.upper_pixel() as u16) / 2) as u8),
            crate::png::PngFilter::Paeth => {
                let A = self.left_pixel() as i16;
                let B = self.upper_pixel() as i16;
                let C = self.left_upper_pixel() as i16;


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
            }
        };

        self.cur_scanline.1 = rem_bytes - 1;

        Ok(Some(corrected))
    }

    pub fn left_pixel(&self) -> u8 {
        if self.cur_scanline.1 >= self.scanline_bytes() - self.source_bytespp as usize {
            return 0;
        }

        self.scanline_buffer[self.source_bytespp as usize - 1]
    }

    pub fn upper_pixel(&self) -> u8 {
        self.scanline_buffer[self.scanline_bytes() - 2]
    }

    pub fn left_upper_pixel(&self) -> u8 {
        if self.cur_scanline.1 >= self.scanline_bytes() - self.source_bytespp as usize {
            return 0;
        }

        self.scanline_buffer[self.source_bytespp as usize + self.scanline_bytes() - 2]
    }

    fn scanline_bytes(&self) -> usize {
        (self.ihdr.width as usize * self.source_bitspp as usize) / 8 + 1
    }
}
