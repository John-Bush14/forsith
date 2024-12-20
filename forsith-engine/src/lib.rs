#![allow(clippy::needless_return)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]


use bindings::{vk_version, VkVersion};
use std::ffi::CString;


pub mod vulkan_app;

pub mod render_target;

//pub(crate) mod command_pool;

pub mod errors;


pub const API_VERSION: fn() -> VkVersion = || vk_version(1, 0, 0);

pub const ENGINE_VERSION: fn() -> VkVersion = || {
    let [major, minor, patch] = env!("CARGO_PKG_VERSION").split('.')
        .map(|num| num.parse::<u32>().expect("incorrect crate version format (incorrect num)"))
        .collect::<Vec<u32>>()
        .try_into().expect("incorrect crate version format (to much nums)");

    return vk_version(major, minor, patch);
};

pub const ENGINE_NAME: fn() -> CString = || CString::new("forsith").unwrap();


pub type DynError = Box<dyn std::error::Error>;


#[cfg(test)]
#[test]
fn test_engine_version() {assert_eq!(ENGINE_VERSION(), vk_version(0, 1, 0))}
