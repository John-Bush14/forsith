#![allow(incomplete_features)]
#![allow(internal_features)]
#![feature(portable_simd)]
#![feature(generic_const_exprs)]
#![feature(const_trait_impl)]
#![feature(const_cmp)]
#![feature(const_precise_live_drops)]
#![feature(generic_const_items)]
#![feature(likely_unlikely)]
#![feature(read_array)]
#![feature(read_le)]
#![feature(integer_widen_truncate)]
#![feature(loop_hints)]
#![feature(stmt_expr_attributes)]
#![feature(option_reference_flattening)]

#![forbid(unsafe_code)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_errors_doc, clippy::inline_always, clippy::option_map_unit_fn)]

// make AdlerLaneSize u32 if increasing SIMD_WIDTH above 16, otherwise u16 is enough
pub const SIMD_WIDTH: usize = 16;

#[cfg(feature = "image")]
pub mod image;

pub mod xml;

pub(crate) use forsith_shared as utils;
pub(crate) use utils::{buffers, int, bit};

#[cfg(feature = "image")]
mod decoding_error;
#[cfg(feature = "image")]
pub use decoding_error::DecodingError;

#[cfg(feature = "image")]
pub(crate) mod parsing;
#[cfg(feature = "image")]
pub(crate) mod checksums;
pub(crate) mod decompression;

