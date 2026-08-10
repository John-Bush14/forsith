use crate::{Channel, DecodingError, buffers::OutputWriter, PixelFormat, PngDecoder, has_alpha, outputconverting::{OutputConverter, get_out_writer_func}, png::{ColorType, chunks::{ColorPalette, Ihdr}}, int::unpack};
use core::simd::prelude::*;
pub use crate::simd::{SIMD_WIDTH, open_simd};

pub const MAX_STRIDE: usize = 8;

impl<C: Channel, const F: u8> PngDecoder<'_, C, F> {
    #[inline(always)]
    #[must_use]
    pub fn scanline_bytes(&self) -> usize {self.postprocessor.scanline_bytes()}
    #[inline(always)]
    #[must_use]
    pub fn scanline_pixel_bytes(&self) -> usize {self.postprocessor.scanline_pixel_bytes()}
    pub fn max_scanline_bytes(&self) -> usize {self.postprocessor.max_scanline_bytes()}
}

pub fn calculate_scanline_bytes(width: u32, bitspp: u8) -> (usize, u8) {
    let scanline_bits = width as usize * bitspp as usize;

    (scanline_bits.div_ceil(8) + 1, ((8 - (scanline_bits % 8)) % 8) as u8)
}

#[derive(Debug)]
pub struct PostProcessor<C: Channel, const F: u8> {
    scanline_bytes: usize,
    max_scanline_bytes: usize,
    stride: usize,
    palette: Option<ColorPalette>,
    color_type: ColorType,
    bitspp: u8,
    scanline_padding: u8, // in bits
    channel_depth: u8,
    out_writer: OutputConverter<C, F>,
    alpha_color: Option<(i64, i64, i64)>,
    adam7_pass: Pass,
}

impl<C: Channel, const F: u8> PostProcessor<C, F> {
    pub fn new(width: u32, color_type: ColorType, channel_depth: u8) -> Self
    {
        let bitspp = PixelFormat::from(color_type) as u8 * channel_depth;

        let (scanline_bytes, scanline_padding) = calculate_scanline_bytes(width, if color_type != ColorType::Indexed {bitspp} else {channel_depth});

        let stride = (bitspp as usize).div_ceil(8);

        let out_format = outputted_pixel_format::<F>(color_type) as u8;
        let out_channel_depth = outputted_channel_depth(channel_depth, color_type);
        let out_writer = get_out_writer_func::<C, F>(out_channel_depth, out_format, false);

        Self {
            scanline_bytes,
            max_scanline_bytes: scanline_bytes,
            stride,
            palette: None,
            color_type,
            bitspp,
            scanline_padding,
            channel_depth,
            out_writer,
            alpha_color: None,
            adam7_pass: Default::default(),
        }
    }

    pub fn filter_inflated_scanline(&mut self, scanlines: &mut [u8], mut i: usize, dest: &mut OutputWriter<'_, C, F>) -> Result<(), DecodingError> {
        let filter = scanlines[i]; i += 1;

        let cur_scanline_range = i..i+self.scanline_pixel_bytes();

        match filter {
            0 => (),
            1 => {self.filter_scanline_inplace::<1>(scanlines, i)},
            2 => {self.filter_scanline_inplace::<2>(scanlines, i)},
            3 => {self.filter_scanline_inplace::<3>(scanlines, i)},
            4 => {self.filter_scanline_inplace::<4>(scanlines, i)},
            _ => {
                println!("{:?} \n\n {:?}", &scanlines[..i-1], &scanlines[i-1..]);

                return Err(DecodingError::InvalidFilter(filter))
            },
        }

        self.emit_filtered_scanline(&scanlines[cur_scanline_range], dest);

        self.scanline_passed(scanlines, i, dest);

        Ok(())
    }

    pub fn emit_filtered_scanline(&mut self, scanline: &[u8], dest: &mut OutputWriter<'_, C, F>) {
        match self.color_type {
            ColorType::Indexed => self.emit_indexed_scanline(scanline, dest),
            _ => self.write_slice(scanline, dest, self.scanline_padding)
        }
    }

    pub fn emit_indexed_scanline(&mut self, scanline: &[u8], dest: &mut OutputWriter<'_, C, F>) {
        let palette = self.palette.as_ref().unwrap();
        let index_bits = self.bitspp / 3;

        let mut push_index = |index: u8| {
            let pixel = palette[index as usize].to_le_bytes();

            if has_alpha(F) {
                self.write_slice(&pixel, dest, 0);
            } else {
                self.write_slice(&pixel[..3], dest, 0);
            }
        };

        match index_bits {
            8 => scanline.iter().cloned().for_each(push_index),
            _ => unpack::<false>(scanline, index_bits, self.scanline_padding, |bs| for &b in bs {push_index(b)})
        }
    }

    #[inline(always)]
    fn write_slice(&self, slice: &[u8], dest: &mut OutputWriter<'_, C, F>, padding: u8) {
        (self.out_writer)(slice, dest, padding, self.alpha_color);
    }

    fn filter_scanline_inplace<const FILTER: u8>(&mut self, scanlines: &mut [u8], i: usize) {
        let mut cur = i;
        let mut up = i - self.scanline_bytes();

        let scanline_bytes = self.handle_border_bytes::<FILTER>(scanlines, &mut cur, &mut up);

        if !should_use_simd::<FILTER>(self.stride) {
            self.filter_inplace_scalar::<FILTER, false>(scanline_bytes, scanlines, &mut cur, &mut up);

            return;
        }

        let alignment_bytes = scanline_bytes % SIMD_WIDTH;

        self.filter_inplace_scalar::<FILTER, false>(alignment_bytes, scanlines, &mut cur, &mut up);

        let simd_iterations = (scanline_bytes - alignment_bytes) / SIMD_WIDTH;
        match self.stride {
            1 => self.filter_inplace_simd::<FILTER, 1>(simd_iterations, scanlines, cur, up),
            2 => self.filter_inplace_simd::<FILTER, 2>(simd_iterations, scanlines, cur, up),
            3 => self.filter_inplace_simd::<FILTER, 3>(simd_iterations, scanlines, cur, up),
            4 => self.filter_inplace_simd::<FILTER, 4>(simd_iterations, scanlines, cur, up),
            6 => self.filter_inplace_simd::<FILTER, 6>(simd_iterations, scanlines, cur, up),
            8 => self.filter_inplace_simd::<FILTER, 8>(simd_iterations, scanlines, cur, up),
            _ => unreachable!()
        };
    }

    #[inline]
    fn handle_border_bytes<const FILTER: u8>(&mut self, scanlines: &mut [u8], cur: &mut usize, up: &mut usize) -> usize {
        if FILTER == 4 {
            scanlines[*up - self.stride..*up].fill(0);
        }

        if FILTER != 2 {
            self.filter_inplace_scalar::<FILTER, true>(self.stride.min(self.scanline_pixel_bytes()), scanlines, cur, up);
            self.scanline_pixel_bytes().saturating_sub(self.stride)
        } else {
            self.scanline_pixel_bytes()
        }
    }

    #[inline]
    fn filter_inplace_scalar<const FILTER: u8, const BORDER: bool>(&mut self, n: usize, scanlines: &mut [u8], cur: &mut usize, up: &mut usize) {
        for _ in 0..n {
            self.filter::<FILTER, BORDER>(scanlines, *cur, *up);
            (*cur, *up) = (*cur + 1, *up + 1);
        };
    }

    fn filter_inplace_simd<const FILTER: u8, const STRIDE: usize>(&mut self, n: usize, scanlines: &mut [u8], mut cur: usize, mut up: usize) {
        for _ in 0..n {
            self.filter_simd::<FILTER, STRIDE>(scanlines, cur, up);

            (cur, up) = (cur + SIMD_WIDTH, up + SIMD_WIDTH);
        }
    }

    #[inline(always)]
    fn filter<const FILTER: u8, const BORDER: bool>(&self, scanlines: &mut [u8], cur: usize, up: usize) {
        let r = match FILTER {
            1 => self.left_byte::<BORDER>(scanlines, cur),
            2 => self.upper_byte(scanlines, up),
            3 => ((self.left_byte::<BORDER>(scanlines, cur) as u16 + self.upper_byte(scanlines, up) as u16) / 2) as u8,
            4 => paeth_predictor(self.left_byte::<BORDER>(scanlines, cur), self.upper_byte(scanlines, up), self.left_upper_byte(scanlines, up)),
            _ => unreachable!(),
        };

        scanlines[cur] = scanlines[cur].wrapping_add(r);
    }

    #[must_use]
    pub fn scanline_bytes(&self) -> usize {self.scanline_bytes}
    #[must_use]
    pub fn scanline_pixel_bytes(&self) -> usize {self.scanline_bytes() - 1}
    pub fn max_scanline_bytes(&self) -> usize {self.max_scanline_bytes}

    pub fn color_type(&self) -> ColorType {self.color_type}

    pub fn stored_bpp(&self) -> usize {
        self.channel_depth as usize * self.stored_format() as usize
    }
    pub fn stored_format(&self) -> PixelFormat {
        match self.color_type() {ColorType::Indexed => PixelFormat::Grayscale, _ => PixelFormat::from(self.color_type)}
    }
    pub fn stored_channel_depth(&self) -> u8 {self.channel_depth}

    #[inline]
    pub fn left_byte<const BORDER: bool>(&self, scanlines: &mut [u8], cur: usize) -> u8 {
        if !BORDER {scanlines[cur - self.stride]} else {0}
    }
    #[inline]
    pub fn upper_byte(&self, scanlines: &mut [u8], up: usize) -> u8 {scanlines[up]}
    #[inline]
    pub fn left_upper_byte(&self, scanlines: &mut [u8], up: usize) -> u8 {scanlines[up - self.stride]}

    pub fn set_palette(&mut self, palette: ColorPalette) {
        self.palette = Some(palette);
    }

    pub fn palette(&self) -> Option<&ColorPalette> {self.palette.as_ref()}
    pub fn palette_mut(&mut self) -> Option<&mut ColorPalette> {self.palette.as_mut()}

    pub fn set_alpha_color(&mut self, c: (i64, i64, i64)) {self.alpha_color = Some(c);}

    pub fn setup_interlacing(&mut self, ihdr: &Ihdr) {
        self.adam7_pass = Pass::new(ihdr, self);
    }
    pub fn update_dest(&self, dest: &mut OutputWriter<'_, C, F>) {
        self.adam7_pass.update_dest(dest)
    }

    #[inline(always)]
    fn scanline_passed(&mut self, scanlines: &mut [u8], i: usize, dest: &mut OutputWriter<'_, C, F>) {
        let pass = &mut self.adam7_pass;

        if pass.end_scanline_skip == 0 {return;}

        pass.cur_scanline += pass.end_scanline_scanlines_passed as usize;

        if pass.cur == 6 && pass.cur_scanline + 1 > pass.dim.1 {return;}
        if pass.cur_scanline >= pass.dim.1 {
            self.next_adam7_pass(scanlines, i, dest);
        } else {
            dest.advance(pass.end_scanline_skip);
        }
    }

    fn next_adam7_pass(&mut self, scanlines: &mut [u8], i: usize, dest: &mut OutputWriter<'_, C, F>) {
        let pass = &mut self.adam7_pass;
        pass.cur += 1;

        let (start, stride, passed_scanlines) = PASSES[pass.cur as usize];
        if start.0 >= pass.dim.0 || start.1 >= pass.dim.1 {return self.scanline_passed(scanlines, i, dest);}

        let width = pass.update(dest, start, stride, passed_scanlines);

        self.update_scanline_bytes(width, scanlines, i);
    }

    fn update_scanline_bytes(&mut self, width: u32, scanlines: &mut [u8], i: usize) {
        let (scanline_bytes, scanline_padding) = calculate_scanline_bytes(width, self.stored_bpp() as u8);

        let end = i + self.scanline_pixel_bytes();
        scanlines[end - self.stride - (scanline_bytes - 1)..end].fill(0);

        self.scanline_bytes = scanline_bytes;
        self.scanline_padding = scanline_padding;
    }
}

const PASSES: [((usize, usize), usize, u8); 7] = [
    ((0, 0), 8, 8),
    ((4, 0), 8, 8),
    ((0, 4), 4, 8),
    ((2, 0), 4, 4),
    ((0, 2), 2, 4),
    ((1, 0), 2, 2),
    ((0, 1), 1, 2)
];

pub fn outputted_pixel_format<const F: u8>(color_type: ColorType) -> PixelFormat {
    if color_type == ColorType::Indexed && has_alpha(F) {
        PixelFormat::TruecolorAlpha
    } else {PixelFormat::from(color_type)}
}
pub fn outputted_channel_depth(channel_depth: u8, color_type: ColorType) -> u8 {
    if color_type == ColorType::Indexed  {8} else {channel_depth}
}

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

#[derive(Debug, Default, Clone)]
pub struct Pass {
    cur: u8,
    stride: usize,
    dim: (usize, usize), // widht, height
    cur_scanline: usize,
    end_scanline_skip: usize,
    end_scanline_scanlines_passed: u8,
}

impl Pass {
    fn update<const F: u8>(&mut self, dest: &mut OutputWriter<'_, impl Channel, F>, start: (usize, usize), stride: usize, passed_scanlines: u8) -> u32 {
        self.cur_scanline = start.1;
        self.end_scanline_scanlines_passed = passed_scanlines;
        self.stride = stride;
        dest.set_stride(self.stride);
        dest.reset(); dest.advance(start.0 + start.1 * self.dim.0);

        let width = ((self.dim.0 - start.0 - 1) as u32).div_euclid(stride as u32) + 1;

        let alignment = (self.dim.0 + start.0) as isize - (start.0 as isize + width as isize * stride as isize);
        self.end_scanline_skip = (((passed_scanlines as usize - 1) * self.dim.0) as isize + alignment) as usize;

        width
    }

    fn new<const F: u8>(ihdr: &Ihdr, postprocessor: &mut PostProcessor<impl Channel, F>) -> Pass {
        let (end_scanline_skip, stride) =  if ihdr.interlace_method == 1 {
            let width = (ihdr.width - 1).div_euclid(8) + 1;
            let alignment = ihdr.width as isize - width as isize * 8;

            let (new_scanline_bytes, padding) = calculate_scanline_bytes(width, ihdr.channel_depth * match ihdr.color_type {ColorType::Indexed => 1, c => PixelFormat::from(c) as u8});
            postprocessor.scanline_padding = padding;

            postprocessor.scanline_bytes = new_scanline_bytes;

            (((ihdr.width as usize * 7) as isize + alignment) as usize, 8)
        } else {
            (0, 1)
        };

        Self {
            cur: 0,
            stride,
            dim: (ihdr.width as usize, ihdr.height as usize),
            cur_scanline: 0,
            end_scanline_skip,
            end_scanline_scanlines_passed: 8
        }
    }

    pub fn update_dest<const F: u8>(&self, dest: &mut OutputWriter<'_, impl Channel, F>) {
        dest.set_stride(self.stride);
    }
}

pub const fn should_use_simd<const FILTER: u8>(stride: usize) -> bool {
    if FILTER == 1 && stride >= 3 {return true}

    if FILTER == 2 {return true}

    // if FILTER == 3 && stride >= 6 {return true}

    false
}

impl<C: Channel, const F: u8> PostProcessor<C, F> {
    #[inline(always)]
    pub fn filter_simd<const FILTER: u8, const STRIDE: usize>(&self, scanlines: &mut [u8], cur: usize, up: usize) {
        let raw_bytes = open_simd(&scanlines[cur..]);

        let result = match FILTER {
            1 => sub_filter::<STRIDE>(raw_bytes, self.left_pixel::<STRIDE>(scanlines, cur)),
            2 => raw_bytes + self.upper_pixels(scanlines, up),
            3 => average_filter::<STRIDE>(raw_bytes, self.left_pixels::<STRIDE>(scanlines, cur), self.upper_pixels(scanlines, up)),
            4 => todo!(),
            _ => unreachable!(),
        };

        scanlines[cur..cur + SIMD_WIDTH].copy_from_slice(result.as_array());
    }

    fn left_pixel<'a, const STRIDE: usize>(&self, scanlines: &'a mut [u8], cur: usize) -> &'a [u8; STRIDE] {
        (&scanlines[cur - STRIDE..cur]).try_into().unwrap()
    }
    /// only first {self.stride} pixels correct, others 0
    fn left_pixels<const STRIDE: usize>(&self, scanlines: &mut [u8], cur: usize) -> Simd<u8, SIMD_WIDTH> {
        let mut left_pixels = Simd::splat(0);
        left_pixels.as_mut_array()[..STRIDE].copy_from_slice(self.left_pixel::<STRIDE>(scanlines, cur));
        left_pixels
    }
    fn upper_pixels(&self, scanlines: &mut [u8], up: usize) -> Simd<u8, SIMD_WIDTH> {
        open_simd(&scanlines[up..])
    }
}

fn average_filter<const STRIDE: usize>(mut raw_bytes: Simd<u8, SIMD_WIDTH>, left_pixels: Simd<u8, SIMD_WIDTH>, mut upper_pixels: Simd<u8, SIMD_WIDTH>) -> Simd<u8, SIMD_WIDTH> {
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
fn sub_filter<const STRIDE: usize>(mut raw_bytes: Simd<u8, SIMD_WIDTH>, left_pixel: &[u8; STRIDE]) -> Simd<u8, SIMD_WIDTH> {
    let mut shifted_bytes = raw_bytes;

    for _ in (STRIDE..SIMD_WIDTH).step_by(STRIDE) {
        shifted_bytes = shifted_bytes.shift_elements_right::<STRIDE>(0);
        raw_bytes += shifted_bytes
    }

    let anchor = array_repeating_to_simd(left_pixel);

    raw_bytes + anchor
}

#[inline]
fn array_repeating_to_simd<const LENGTH: usize>(arr: &[u8; LENGTH]) -> Simd<u8, SIMD_WIDTH> {
    Simd::<u8, LENGTH>::from_slice(arr).resize::<{SIMD_WIDTH}>(0).swizzle_dyn(Simd::from_array(repeating_swizzle_index::<{LENGTH}>()))
}

const fn repeating_swizzle_index<const MAX_INDEX: usize>() -> [u8; SIMD_WIDTH] {
    let mut arr = [0; SIMD_WIDTH];
    let mut i = 0;

    while i < SIMD_WIDTH {
        arr[i] = (i % MAX_INDEX) as u8;
        i += 1;
    }

    arr
}
