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

pub mod image;

pub mod xml;

mod utils;
pub(crate) use utils::{buffers, int, simd};

mod decoding_error;
pub use decoding_error::DecodingError;

pub(crate) mod parsing;
pub(crate) mod checksums;
pub(crate) mod decompression;

