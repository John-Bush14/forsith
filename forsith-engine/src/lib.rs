use bindings::{vk_version, VkVersion};


pub mod vulkan_app;


#[allow(dead_code)]
pub const API_VERSION: fn() -> VkVersion = || vk_version(1, 0, 0);


pub type DynError = Box<dyn std::error::Error>;
