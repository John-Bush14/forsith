use crate::image::{PixelFormat, PngDecoder, outputconverting::OutputWriter, png::chunks::Ihdr};

const PASSES: [((usize, usize), usize, u8); 7] = [
    ((0, 0), 8, 8),
    ((4, 0), 8, 8),
    ((0, 4), 4, 8),
    ((2, 0), 4, 4),
    ((0, 2), 2, 4),
    ((1, 0), 2, 2),
    ((0, 1), 1, 2),
];

#[derive(Debug, Default, Clone)]
pub struct Adam7Pass {
    cur: u8,
    stride: usize,
    dim: (usize, usize), // widht, height
    cur_scanline: usize,
    end_scanline_skip: usize,
    end_scanline_scanlines_passed: u8,
    cur_scanline_bytes: usize,
    max_scanline_bytes: usize,
    scanline_padding: u8, // in bits
}

impl PngDecoder<'_> {
    #[must_use]
    pub const fn scanline_bytes(&self) -> usize {
        self.pass.scanline_bytes()
    }

    #[must_use]
    pub const fn max_scanline_bytes(&self) -> usize {
        self.pass.max_scanline_bytes()
    }

    #[must_use]
    pub const fn scanline_pixel_bytes(&self) -> usize {
        self.pass.scanline_pixel_bytes()
    }
}

impl Adam7Pass {
    pub const fn scanline_bytes(&self) -> usize {
        self.cur_scanline_bytes
    }

    pub const fn max_scanline_bytes(&self) -> usize {
        self.max_scanline_bytes
    }

    #[inline(always)]
    pub fn scanline_passed(
        &mut self,
        scanlines: &mut [u8],
        i: usize,
        bpp: u8,
        dest: &mut OutputWriter<'_>,
    ) {
        if self.end_scanline_skip == 0 {
            return;
        }

        self.cur_scanline += self.end_scanline_scanlines_passed as usize;

        if self.cur == 6 && self.cur_scanline + 1 > self.dim.1 {
            return;
        }
        if self.cur_scanline >= self.dim.1 {
            self.next_adam7_pass(scanlines, i, bpp, dest);
        } else {
            dest.advance(self.end_scanline_skip);
        }
    }

    fn next_adam7_pass(
        &mut self,
        scanlines: &mut [u8],
        i: usize,
        bpp: u8,
        dest: &mut OutputWriter<'_>,
    ) {
        self.cur += 1;

        let (start, stride, passed_scanlines) = PASSES[self.cur as usize];
        if start.0 >= self.dim.0 || start.1 >= self.dim.1 {
            return self.scanline_passed(scanlines, i, bpp, dest);
        }

        let width = self.update(dest, start, stride, passed_scanlines);

        self.update_scanline_bytes(width, scanlines, i, bpp);
    }

    pub const fn scanline_pixel_bytes(&self) -> usize {
        self.cur_scanline_bytes - 1
    }

    #[allow(clippy::cast_possible_truncation)]
    fn update_scanline_bytes(&mut self, width: u32, scanlines: &mut [u8], i: usize, bpp: u8) {
        let (scanline_bytes, scanline_padding) = calculate_scanline_bytes(width, bpp);

        let end = i + self.scanline_pixel_bytes();
        scanlines[end - self.stride - (scanline_bytes - 1)..end].fill(0);

        self.cur_scanline_bytes = scanline_bytes;
        self.scanline_padding = scanline_padding;
    }

    fn update(
        &mut self,
        dest: &mut OutputWriter<'_>,
        start: (usize, usize),
        stride: usize,
        passed_scanlines: u8,
    ) -> u32 {
        self.cur_scanline = start.1;
        self.end_scanline_scanlines_passed = passed_scanlines;
        self.stride = stride;
        self.update_dest(dest);
        dest.reset();
        dest.advance(start.0 + start.1 * self.dim.0);

        let width = (self.dim.0 - start.0 - 1).div_euclid(stride) + 1;

        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_possible_wrap
        )]
        {
            let alignment = (self.dim.0 + start.0) as isize
                - (start.0 as isize + width as isize * stride as isize);

            self.end_scanline_skip =
                (((passed_scanlines as usize - 1) * self.dim.0) as isize + alignment) as usize;
        }

        u32::try_from(width).unwrap()
    }

    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_possible_wrap
    )]
    pub const fn new(ihdr: &Ihdr, format: PixelFormat) -> Self {
        let (max_scanline_bytes, scanline_padding) =
            calculate_scanline_bytes(ihdr.width, format.bpp());

        let (scanline_bytes, scanline_padding, end_scanline_skip, stride) =
            if ihdr.interlace_method == 1 {
                let width = (ihdr.width - 1).div_euclid(8) + 1;
                let alignment = ihdr.width as isize - width as isize * 8;
                let end_scanline_skip = ((ihdr.width as usize * 7) as isize + alignment) as usize;

                let (new_scanline_bytes, padding) = calculate_scanline_bytes(width, format.bpp());

                (new_scanline_bytes, padding, end_scanline_skip, 8)
            } else {
                (max_scanline_bytes, scanline_padding, 0, 1)
            };

        Self {
            cur: 0,
            stride,
            dim: (ihdr.width as usize, ihdr.height as usize),
            cur_scanline: 0,
            end_scanline_skip,
            end_scanline_scanlines_passed: 8,
            cur_scanline_bytes: scanline_bytes,
            max_scanline_bytes,
            scanline_padding,
        }
    }

    pub const fn update_dest(&self, dest: &mut OutputWriter<'_>) {
        dest.set_stride(self.stride);
    }
}

#[allow(clippy::cast_possible_truncation)]
pub const fn calculate_scanline_bytes(width: u32, bitspp: u8) -> (usize, u8) {
    let scanline_bits = width as usize * bitspp as usize;

    (
        scanline_bits.div_ceil(8) + 1,
        ((8 - (scanline_bits % 8)) % 8) as u8,
    )
}
