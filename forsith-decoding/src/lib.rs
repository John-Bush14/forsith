#![allow(incomplete_features)]
#![feature(portable_simd)]
#![feature(integer_casts)]
#![feature(generic_const_exprs)]
#![feature(const_trait_impl)]
#![feature(const_cmp)]
#![feature(const_precise_live_drops)]
#![feature(const_try)]
#![feature(f16)]
#![feature(float_bits_const)]

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
const fn is_rgb(format: u8) -> bool {matches!(format, 3 | 4)}
const fn is_gray(format: u8) -> bool {matches!(format, 1 | 2)}

pub trait ImageDecoder<'a, R: Read, C: Channel, const F: u8> {
    fn dest_bitspp(&self) -> u8 {C::BIT_DEPTH * F}

    fn open_validated(data: R) -> Result<Self, DecodingError> where Self: Sized;
    fn open(data: R) -> Result<Self, DecodingError> where Self: Sized {
        assert!((C::BIT_DEPTH * F).is_multiple_of(8));
        assert!(1 <= F && F <= 4);

        Self::open_validated(data)
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, DecodingError>;

    fn bit_depth(&self) -> u8;
    fn pixel_format(&self) -> PixelFormat;
}
impl<R: Read, C: Channel, const F: u8> Read for dyn ImageDecoder<'_, R, C, F> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {self.read(buf).map_err(io::Error::other)}
}

pub enum ChannelType {
    Unsigned,
    Signed,
    // Float,
    // NormalizedFloat
}

pub trait Channel {
    type StorageType;
    const BIT_DEPTH: u8;
    const MAX: Self::StorageType;
    const TYPE: ChannelType;
}

impl<I: Int> Channel for I {
    type StorageType = I;
    const BIT_DEPTH: u8 = I::BIT_DEPTH;
    const MAX: I = I::MAX;
    const TYPE: ChannelType = {
        match I::SIGNED {
            true => ChannelType::Signed,
            false => ChannelType::Unsigned
        }
    };
}

pub struct U4 {}
impl Channel for U4 {
    type StorageType = u8;
    const BIT_DEPTH: u8 = 4;
    const MAX: u8 = 15;
    const TYPE: ChannelType = ChannelType::Unsigned;
}

pub struct I4 {}
impl Channel for I4 {
    type StorageType = u8;
    const BIT_DEPTH: u8 = 4;
    const MAX: u8 = 7;
    const TYPE: ChannelType = ChannelType::Unsigned;
}
