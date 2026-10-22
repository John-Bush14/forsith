use crate::image::{
    PixelFormat, PngDecoder,
    outputconverting::OutputWriter,
    png::{chunks::Ihdr, filtering::ScanlinePair},
};

struct InterlacePass {
    start: (usize, usize),
    stride: usize,
    passed_scanlines: u8,
}

impl InterlacePass {
    const fn new(start: (usize, usize), stride: usize, passed_scanlines: u8) -> Self {
        Self {
            start,
            stride,
            passed_scanlines,
        }
    }
}

const PASSES: [InterlacePass; 7] = [
    InterlacePass::new((0, 0), 8, 8),
    InterlacePass::new((4, 0), 8, 8),
    InterlacePass::new((0, 4), 4, 8),
    InterlacePass::new((2, 0), 4, 4),
    InterlacePass::new((0, 2), 2, 4),
    InterlacePass::new((1, 0), 2, 2),
    InterlacePass::new((0, 1), 1, 2),
];

#[derive(Debug, Default, Clone)]
pub struct InterlaceState {
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
        self.interlace.scanline_bytes()
    }

    #[must_use]
    pub const fn max_scanline_bytes(&self) -> usize {
        self.interlace.max_scanline_bytes()
    }

    #[must_use]
    pub const fn scanline_pixel_bytes(&self) -> usize {
        self.interlace.scanline_pixel_bytes()
    }
}

impl InterlaceState {
    pub const fn scanline_bytes(&self) -> usize {
        self.cur_scanline_bytes
    }

    pub const fn max_scanline_bytes(&self) -> usize {
        self.max_scanline_bytes
    }

    pub const fn interlacing_enabled(&self) -> bool {
        self.end_scanline_skip > 0
    }

    #[inline(always)]
    pub fn scanline_passed(
        &mut self,
        scanlines: ScanlinePair<'_>,
        bpp: u8,
        dest: &mut OutputWriter<'_>,
    ) {
        if !self.interlacing_enabled() {
            return;
        }

        self.cur_scanline += self.end_scanline_scanlines_passed as usize;

        if self.cur == 6 && self.cur_scanline + 1 > self.dim.1 {
            return;
        }

        if self.cur_scanline >= self.dim.1 {
            return self.next_adam7_pass(scanlines, bpp, dest);
        }

        dest.advance(self.end_scanline_skip);
    }

    fn next_adam7_pass(
        &mut self,
        scanlines: ScanlinePair<'_>,
        bpp: u8,
        dest: &mut OutputWriter<'_>,
    ) {
        self.cur += 1;

        let pass = &PASSES[self.cur as usize];
        if pass.start.0 >= self.dim.0 || pass.start.1 >= self.dim.1 {
            return self.scanline_passed(scanlines, bpp, dest);
        }

        self.update_state(pass, bpp, Some(scanlines));
        self.update_dest_state(dest, Some(pass));
    }

    pub const fn scanline_pixel_bytes(&self) -> usize {
        self.cur_scanline_bytes - 1
    }

    fn update_state(&mut self, pass: &InterlacePass, bpp: u8, scanlines: Option<ScanlinePair<'_>>) {
        let width = (self.dim.0 - pass.start.0 - 1).div_euclid(pass.stride) + 1;

        self.cur_scanline = pass.start.1;
        self.end_scanline_scanlines_passed = pass.passed_scanlines;
        self.stride = pass.stride;
        self.end_scanline_skip =
            usize::from(pass.passed_scanlines) * self.dim.0 - width * pass.stride;

        let (scanline_bytes, scanline_padding) = calculate_scanline_bytes(width, bpp);

        if let Some(mut scanlines) = scanlines {
            let end = scanlines.both_scanlines().len();
            scanlines[end - self.stride - (scanline_bytes - 1)..end].fill(0);
        }

        self.cur_scanline_bytes = scanline_bytes;
        self.scanline_padding = scanline_padding;
    }

    pub fn new(ihdr: &Ihdr, format: PixelFormat) -> Self {
        let (max_scanline_bytes, scanline_padding) =
            calculate_scanline_bytes(usize::try_from(ihdr.width).unwrap(), format.bpp());

        let mut state = Self {
            cur: 0,
            stride: 1,
            dim: (ihdr.width as usize, ihdr.height as usize),
            cur_scanline: 0,
            end_scanline_skip: 0,
            end_scanline_scanlines_passed: 8,
            cur_scanline_bytes: max_scanline_bytes,
            max_scanline_bytes,
            scanline_padding,
        };

        if ihdr.interlace_method == 1 {
            state.update_state(&PASSES[0], format.bpp(), None);
        }

        state
    }

    pub const fn update_dest(&self, dest: &mut OutputWriter<'_>) {
        dest.set_stride(self.stride);
    }

    const fn update_dest_state(&self, dest: &mut OutputWriter<'_>, pass: Option<&InterlacePass>) {
        dest.set_stride(self.stride);

        if let Some(pass) = pass {
            dest.reset();
            dest.advance(pass.start.0 + pass.start.1 * self.dim.0);
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
pub const fn calculate_scanline_bytes(width: usize, bitspp: u8) -> (usize, u8) {
    let scanline_bits = width * bitspp as usize;

    (
        scanline_bits.div_ceil(8) + 1,
        ((8 - (scanline_bits % 8)) % 8) as u8,
    )
}
