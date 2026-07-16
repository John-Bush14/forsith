#![allow(incomplete_features)]
#![feature(portable_simd)]
#![feature(integer_casts)]
#![feature(generic_const_exprs)]
#![feature(const_trait_impl)]
#![feature(const_cmp)]
#![feature(const_precise_live_drops)]
#![feature(const_try)]

use std::{io::{self, Read}};
use num_enum::{IntoPrimitive, TryFromPrimitive};

mod png;
pub use png::PngDecoder;

// don't ask me
include!("buffers.rs");

mod decoding_error;
pub use decoding_error::DecodingError;

// if you can use ['use'] without tanking performance please do
include!("num.rs");


#[repr(u8)]
#[derive(TryFromPrimitive, IntoPrimitive)]
pub enum PixelFormat {
    Grayscale = 1,
    Truecolor = 3,
    GrayscaleAlpha = 2,
    TruecolorAlpha = 4
}

const fn has_alpha(format: u8) -> bool {matches!(format, 2 | 4)}

pub trait ImageDecoder<'a, R: Read, const D: u8, const F: u8> {
    fn dest_bitspp(&self) -> u8 {D * F}
    fn dest_bytespp(&self) -> u8 {D*F / 8}

    fn open(data: R) -> Result<Self, DecodingError> where Self: Sized;

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, DecodingError>;

    fn bit_depth(&self) -> u8;
    fn pixel_format(&self) -> PixelFormat;
}
impl<R: Read, const D: u8, const F: u8> Read for dyn ImageDecoder<'_, R, D, F> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {self.read(buf).map_err(io::Error::other)}
}
