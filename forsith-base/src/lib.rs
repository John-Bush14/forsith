#![cfg_attr(not(feature = "std"), no_std)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]

pub(crate) extern crate alloc;

pub mod casing;

pub mod proc_macro;

pub mod bitvec;

pub mod buffer;

pub mod bitflags;
