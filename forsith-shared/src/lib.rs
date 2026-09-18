#![cfg_attr(not(feature = "std"), no_std)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::inline_always, clippy::missing_errors_doc)]

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;
#[cfg(feature = "std")]
pub use std as alloc;

#[cfg(feature = "std")]
pub mod interner;

#[cfg(feature = "alloc")]
pub mod arena;

#[cfg(feature = "std")]
pub mod buffers;

pub mod int;

#[cfg(feature = "std")]
pub mod bit;

#[cfg(feature = "alloc")]
pub mod error;

#[cfg(feature = "std")]
pub mod ffi;

pub mod rng;

pub use forsith_base::{casing, proc_macro};
