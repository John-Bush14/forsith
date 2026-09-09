#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::inline_always, clippy::missing_errors_doc)]

pub mod interner;

pub mod arena;

pub mod buffers;

pub mod int;

pub mod bit;

pub mod error;

pub mod ffi;

pub use forsith_base::{
    casing,
    proc_macro,
};
