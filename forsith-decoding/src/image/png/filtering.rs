use crate::{DecodingError, SIMD_WIDTH};
use core::simd::Simd;

pub struct InplaceDefilterer<'a> {
    scanlines: &'a mut [u8],
    stride: usize,
    scanline_bytes: usize,
    start: usize,
}

impl<'a> InplaceDefilterer<'a> {
    pub fn open(
        buffer: &'a mut [u8],
        stride: usize,
        start: usize,
        scanline_bytes: usize,
        prev_scanline_bytes: usize,
    ) -> Self {
        let scanlines = &mut buffer[start - prev_scanline_bytes..start + scanline_bytes];

        Self {
            scanlines,
            stride,
            scanline_bytes,
            start: prev_scanline_bytes,
        }
    }

    pub const fn both_scanlines(&mut self) -> &mut [u8] {
        self.scanlines
    }

    pub fn filtered_scanline(&mut self) -> &mut [u8] {
        &mut self.scanlines[self.start..self.start + self.scanline_bytes - 1]
    }

    pub fn defilter(&mut self) -> Result<(), DecodingError> {
        let filter = self.scanlines[self.start];
        self.start += 1;

        match filter {
            0 => (),
            1 => self.defilter_inplace::<1>(),
            2 => self.defilter_inplace::<2>(),
            3 => self.defilter_inplace::<3>(),
            4 => self.defilter_inplace::<4>(),
            _ => return Err(DecodingError::InvalidFilter(filter)),
        }

        Ok(())
    }

    fn defilter_inplace<const FILTER: u8>(&mut self) {
        let mut cur = self.start;
        let mut up = self.start - self.scanline_bytes();

        let scanline_bytes = self.handle_border_bytes::<FILTER>(&mut cur, &mut up);

        if !should_use_simd::<FILTER>(self.stride) {
            self.filter_inplace_scalar::<FILTER, false>(scanline_bytes, &mut cur, &mut up);

            return;
        }

        let alignment_bytes = scanline_bytes % SIMD_WIDTH;

        self.filter_inplace_scalar::<FILTER, false>(alignment_bytes, &mut cur, &mut up);

        let simd_iterations = (scanline_bytes - alignment_bytes) / SIMD_WIDTH;
        match self.stride {
            1 => self.filter_inplace_simd::<FILTER, 1>(simd_iterations, cur, up),
            2 => self.filter_inplace_simd::<FILTER, 2>(simd_iterations, cur, up),
            3 => self.filter_inplace_simd::<FILTER, 3>(simd_iterations, cur, up),
            4 => self.filter_inplace_simd::<FILTER, 4>(simd_iterations, cur, up),
            6 => self.filter_inplace_simd::<FILTER, 6>(simd_iterations, cur, up),
            8 => self.filter_inplace_simd::<FILTER, 8>(simd_iterations, cur, up),
            _ => unreachable!(),
        }
    }

    #[inline]
    pub const fn left_byte<const BORDER: bool>(&self, cur: usize) -> u8 {
        if BORDER {
            0
        } else {
            self.scanlines[cur - self.stride]
        }
    }
    #[inline]
    #[allow(clippy::unused_self)]
    pub const fn upper_byte(&self, up: usize) -> u8 {
        self.scanlines[up]
    }
    #[inline]
    pub const fn left_upper_byte(&self, up: usize) -> u8 {
        self.scanlines[up - self.stride]
    }

    #[inline]
    fn handle_border_bytes<const FILTER: u8>(&mut self, cur: &mut usize, up: &mut usize) -> usize {
        if FILTER == 4 {
            self.scanlines[*up - self.stride..*up].fill(0);
        }

        if FILTER == 2 {
            self.scanline_pixel_bytes()
        } else {
            self.filter_inplace_scalar::<FILTER, true>(
                self.stride.min(self.scanline_pixel_bytes()),
                cur,
                up,
            );
            self.scanline_pixel_bytes().saturating_sub(self.stride)
        }
    }

    #[inline]
    fn filter_inplace_scalar<const FILTER: u8, const BORDER: bool>(
        &mut self,
        n: usize,
        cur: &mut usize,
        up: &mut usize,
    ) {
        for _ in 0..n {
            self.filter::<FILTER, BORDER>(*cur, *up);
            (*cur, *up) = (*cur + 1, *up + 1);
        }
    }

    fn filter_inplace_simd<const FILTER: u8, const STRIDE: usize>(
        &mut self,
        n: usize,
        mut cur: usize,
        mut up: usize,
    ) {
        for _ in 0..n {
            self.filter_simd::<FILTER, STRIDE>(cur, up);

            (cur, up) = (cur + SIMD_WIDTH, up + SIMD_WIDTH);
        }
    }

    #[inline(always)]
    fn filter<const FILTER: u8, const BORDER: bool>(&mut self, cur: usize, up: usize) {
        let r = match FILTER {
            1 => self.left_byte::<BORDER>(cur),
            2 => self.upper_byte(up),
            3 => u8::midpoint(self.left_byte::<BORDER>(cur), self.upper_byte(up)),
            4 => paeth_predictor(
                self.left_byte::<BORDER>(cur),
                self.upper_byte(up),
                self.left_upper_byte(up),
            ),
            _ => unreachable!(),
        };

        self.scanlines[cur] = self.scanlines[cur].wrapping_add(r);
    }

    const fn scanline_pixel_bytes(&self) -> usize {
        self.scanline_bytes - 1
    }

    const fn scanline_bytes(&self) -> usize {
        self.scanline_bytes
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
const fn paeth_predictor(a: u8, b: u8, c: u8) -> u8 {
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

pub const fn should_use_simd<const FILTER: u8>(stride: usize) -> bool {
    if FILTER == 1 && stride >= 3 {
        return true;
    }

    if FILTER == 2 {
        return true;
    }

    // if FILTER == 3 && stride >= 6 {return true}

    false
}

#[inline(always)]
pub const fn open_simd(slice: &[u8]) -> Simd<u8, { SIMD_WIDTH }> {
    Simd::from_slice(slice)
}

impl InplaceDefilterer<'_> {
    #[inline(always)]
    pub fn filter_simd<const FILTER: u8, const STRIDE: usize>(&mut self, cur: usize, up: usize) {
        let raw_bytes = open_simd(&self.scanlines[cur..]);

        let result = match FILTER {
            1 => sub_filter::<STRIDE>(raw_bytes, self.left_pixel::<STRIDE>(cur)),
            2 => raw_bytes + self.upper_pixels(up),
            3 => average_filter::<STRIDE>(
                raw_bytes,
                self.left_pixels::<STRIDE>(cur),
                self.upper_pixels(up),
            ),
            4 => todo!(),
            _ => unreachable!(),
        };

        self.scanlines[cur..cur + SIMD_WIDTH].copy_from_slice(result.as_array());
    }

    #[allow(clippy::unused_self)]
    fn left_pixel<const STRIDE: usize>(&self, cur: usize) -> &[u8; STRIDE] {
        (&self.scanlines[cur - STRIDE..cur]).try_into().unwrap()
    }
    /// only first {self.stride} pixels correct, others 0
    fn left_pixels<const STRIDE: usize>(&self, cur: usize) -> Simd<u8, SIMD_WIDTH> {
        let mut left_pixels = Simd::splat(0);
        left_pixels.as_mut_array()[..STRIDE].copy_from_slice(self.left_pixel::<STRIDE>(cur));
        left_pixels
    }
    #[allow(clippy::unused_self)]
    fn upper_pixels(&self, up: usize) -> Simd<u8, SIMD_WIDTH> {
        open_simd(&self.scanlines[up..])
    }
}

fn average_filter<const STRIDE: usize>(
    mut raw_bytes: Simd<u8, SIMD_WIDTH>,
    left_pixels: Simd<u8, SIMD_WIDTH>,
    mut upper_pixels: Simd<u8, SIMD_WIDTH>,
) -> Simd<u8, SIMD_WIDTH> {
    raw_bytes += simd_average(left_pixels, upper_pixels);

    let mut shifted_bytes = raw_bytes;

    for _ in (STRIDE..SIMD_WIDTH).step_by(STRIDE) {
        shifted_bytes = shifted_bytes.shift_elements_right::<STRIDE>(0);
        upper_pixels = upper_pixels.shift_elements_right::<STRIDE>(0);

        raw_bytes += simd_average(shifted_bytes, upper_pixels);
    }

    raw_bytes
}

fn simd_average(a: Simd<u8, SIMD_WIDTH>, b: Simd<u8, SIMD_WIDTH>) -> Simd<u8, SIMD_WIDTH> {
    (a & b) + ((a ^ b) >> Simd::splat(1))
}

#[inline]
fn sub_filter<const STRIDE: usize>(
    mut raw_bytes: Simd<u8, SIMD_WIDTH>,
    left_pixel: &[u8; STRIDE],
) -> Simd<u8, SIMD_WIDTH> {
    let mut shifted_bytes = raw_bytes;

    for _ in (STRIDE..SIMD_WIDTH).step_by(STRIDE) {
        shifted_bytes = shifted_bytes.shift_elements_right::<STRIDE>(0);
        raw_bytes += shifted_bytes;
    }

    let anchor = array_repeating_to_simd(left_pixel);

    raw_bytes + anchor
}

#[inline]
fn array_repeating_to_simd<const LENGTH: usize>(arr: &[u8; LENGTH]) -> Simd<u8, SIMD_WIDTH> {
    Simd::<u8, LENGTH>::from_slice(arr)
        .resize::<{ SIMD_WIDTH }>(0)
        .swizzle_dyn(Simd::from_array(repeating_swizzle_index::<{ LENGTH }>()))
}

#[allow(clippy::cast_possible_truncation)]
const fn repeating_swizzle_index<const MAX_INDEX: usize>() -> [u8; SIMD_WIDTH] {
    let mut arr = [0; SIMD_WIDTH];
    let mut i = 0;

    while i < SIMD_WIDTH {
        arr[i] = (i % MAX_INDEX) as u8;
        i += 1;
    }

    arr
}
