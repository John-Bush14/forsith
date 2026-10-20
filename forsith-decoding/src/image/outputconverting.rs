use crate::image::PixelFormat;

pub struct OutputWriter<'a> {
    buffer: &'a mut [u8],
    index: usize,
    full: bool,
    stride: usize,
    bytespp: u8,
}

impl<'a> OutputWriter<'a> {
    pub const fn new(buffer: &'a mut [u8], format: PixelFormat) -> Self {
        Self {
            buffer,
            index: 0,
            full: false,
            stride: 1,
            bytespp: format
                .bpp()
                .checked_div(8)
                .expect("PixelFormat bpp must be a multiple of 8"),
        }
    }

    #[inline(always)]
    pub const fn push_channel(&mut self, c: u8) {
        self.buffer[self.index] = c;
        self.index += 1;
    }

    pub const fn remaining(&self) -> usize {
        self.buffer.len().saturating_sub(self.index)
    }

    pub const fn set_stride(&mut self, pixels: usize) {
        self.stride = (pixels - 1) * self.bytespp as usize;
    }
    #[inline(always)]
    pub const fn pushed_pixel(&mut self) {
        self.index += self.stride;
    }
    #[inline(always)]
    pub const fn advance(&mut self, pixels: usize) {
        self.index += pixels * self.bytespp as usize;
    }
    pub const fn reset(&mut self) {
        self.index = 0;
    }

    pub const fn len(&self) -> usize {
        self.index
    }

    pub const fn is_full(&self) -> bool {
        self.full
    }
    pub const fn set_full(&mut self) {
        self.full = true;
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub const fn capacity(&self) -> usize {
        self.buffer.len()
    }

    pub fn remaining_mut_slice(&mut self) -> &mut [u8] {
        &mut self.buffer[self.index..]
    }

    pub fn write_slice(&mut self, slice: &[u8]) {
        if self.stride == 0 {
            self.buffer[self.index..self.index + slice.len()].copy_from_slice(slice);
            self.index += slice.len();
        } else {
            for pixel in slice.chunks(self.bytespp as usize) {
                self.buffer[self.index..self.index + pixel.len()].copy_from_slice(pixel);
                self.index += pixel.len() + self.stride;
            }
        }
    }
}
