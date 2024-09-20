use bindings::VkVersion;


pub type DynError = Box<dyn std::error::Error>;


mod drop;

#[allow(non_snake_case)]
mod E2E_tests;

pub mod drawable;
pub mod render_target;
pub mod vulkan_app;
pub mod world_view;
pub mod uniform_object;


pub const ENGINE_NAME: &str = "forsith";

pub const ENGINE_VERSION: fn() -> VkVersion = || vk_make_version(0, 1, 0);
pub const API_VERSION: fn() -> VkVersion = || vk_make_version(1, 0, 0);


pub fn vk_make_version(major: u32, minor: u32, patch: u32) -> VkVersion {
    return (major << 22) | (minor << 12) | patch;
}
