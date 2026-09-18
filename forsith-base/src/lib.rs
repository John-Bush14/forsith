#![no_std]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
pub mod casing;

#[cfg(feature = "alloc")]
pub mod proc_macro;

pub mod bitvec;

#[cfg(feature = "alloc")]
pub mod buffer;

pub mod bitflags;
