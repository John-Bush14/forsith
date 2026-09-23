#![feature(portable_simd, likely_unlikely)]
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::inline_always, clippy::missing_errors_doc)]

pub extern crate alloc;

pub mod interner;

pub mod arena;

#[cfg(feature = "std")]
pub mod buffers;

pub mod hashing;

pub mod collections;

pub mod int;

#[cfg(feature = "std")]
pub mod bit;

pub mod error;

// #[cfg(feature = "std")]
// pub mod ffi;

pub mod rng;

pub use forsith_base::{casing, proc_macro};
