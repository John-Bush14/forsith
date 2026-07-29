#![allow(incomplete_features)]
#![allow(internal_features)]
#![feature(portable_simd)]
#![feature(generic_const_exprs)]
#![feature(const_trait_impl)]
#![feature(const_cmp)]
#![feature(const_precise_live_drops)]
#![feature(const_try)]
#![feature(generic_const_items)]
#![feature(likely_unlikely)]
#![feature(read_array)]
#![feature(read_le)]

#![forbid(unsafe_code)]

use std::io::Read;
use num_enum::{IntoPrimitive, TryFromPrimitive};

mod png;
pub use png::PngDecoder;

// don't ask me
include!("buffers.rs");

mod decoding_error;
pub use decoding_error::DecodingError;

// if you can use ['use'] without tanking performance please do
include!("int.rs");

mod outputconverting;

pub(crate) mod compression;

#[repr(u8)]
#[derive(TryFromPrimitive, IntoPrimitive)]
pub enum PixelFormat {
    Native = 0,
    Grayscale = 1,
    Truecolor = 3,
    GrayscaleAlpha = 2,
    TruecolorAlpha = 4
}

const fn has_alpha(format: u8) -> bool {matches!(format, 2 | 4)}
const fn is_rgb(format: u8) -> bool {matches!(format, 3 | 4)}
const fn is_gray(format: u8) -> bool {matches!(format, 1 | 2)}

pub(crate) const fn bitspp<C: Channel, const F: u8>() -> u8 {C::BIT_DEPTH * F}
pub(crate) const fn bytespp<C: Channel, const F: u8>() -> u8 {C::BIT_DEPTH * F / 8}

pub trait ImageDecoder<'a, R: Read, C: Channel, const F: u8> {
    fn open_validated(data: R) -> Result<Self, DecodingError> where Self: Sized;
    fn open(data: R) -> Result<Self, DecodingError> where Self: Sized {
        assert!((C::BIT_DEPTH * F).is_multiple_of(8));
        assert!(PixelFormat::try_from(F).is_ok());

        Self::open_validated(data)
    }

    fn read(&mut self, buf: &mut [C::StorageType]) -> Result<usize, DecodingError>;

    fn image_dimensions(&self) -> (usize, usize);

    fn min_buf_size(&self) -> usize;
    fn max_buf_size(&self) -> usize {
        let dim = self.image_dimensions();
        dim.0 * dim.1 * F as usize
    }

    fn source_bit_depth(&self) -> u8;
    fn source_pixel_format(&self) -> PixelFormat;
}

#[derive(Debug, PartialEq, Eq)]
pub enum ChannelType {
    Unsigned,
    Signed,
    // Float,
    // NormalizedFloat
}

pub trait Channel
where
    <Self::StorageType as TryFrom<i64>>::Error: Debug,
    <Self::StorageType as TryFrom<u64>>::Error: Debug
{
    type StorageType: Int;
    const BIT_DEPTH: u8;
    const MAX: u64;
    const MIN: i64;
    const TYPE: ChannelType;
}

impl<I: Int> Channel for I
where
    <I as TryFrom<i64>>::Error: Debug,
    <I as TryFrom<u64>>::Error: Debug
{
    type StorageType = I;
    const BIT_DEPTH: u8 = I::BIT_DEPTH;
    const MAX: u64 = I::MAX;
    const MIN: i64 = I::MIN;
    const TYPE: ChannelType = {
        match I::SIGNED {
            true => ChannelType::Signed,
            false => ChannelType::Unsigned
        }
    };
}
