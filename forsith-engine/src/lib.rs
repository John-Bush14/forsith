use bindings::{vk_version, VkVersion};
use std::ffi::CString;


pub mod vulkan_app;


pub const API_VERSION: fn() -> VkVersion = || vk_version(1, 0, 0);

pub const ENGINE_VERSION: fn() -> VkVersion = || {
    let [major, minor, patch] = env!("CARGO_PKG_VERSION").split('.')
        .map(|num| return num.parse::<u32>().expect("incorrect crate version format (incorrect num)"))
        .collect::<Vec<u32>>()
        .try_into().expect("incorrect crate version format (to much nums)");

    return vk_version(major, minor, patch);
};

pub const ENGINE_NAME: fn() -> CString = || CString::new("forsith").unwrap();


pub type DynError = Box<dyn std::error::Error>;
