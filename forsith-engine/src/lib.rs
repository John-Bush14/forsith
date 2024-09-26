use bindings::{vk_version, VkVersion};
use std::ffi::CString;


pub mod vulkan_app;


pub const API_VERSION: fn() -> VkVersion = || vk_version(1, 0, 0);

pub const ENGINE_VERSION: fn() -> VkVersion = || vk_version(0, 1, 0);

pub const ENGINE_NAME: fn() -> CString = || CString::new("forsith").unwrap();


pub type DynError = Box<dyn std::error::Error>;
