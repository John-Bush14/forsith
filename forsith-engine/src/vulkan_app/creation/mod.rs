use bindings::VkVersion;
use crate::DynError;
use super::VulkanApp;
use bindings::vk_version;


#[allow(dead_code)]
pub const API_VERSION: fn() -> VkVersion = || vk_version(1, 0, 0);


#[allow(dead_code)]
impl VulkanApp {
    fn new(_app_name: &str, _app_version: VkVersion) -> Result<Self, DynError> {todo!();}
}


#[cfg(test)]
#[test]
fn vulkan_app_creation_test() {VulkanApp::new("test", vk_version(0, 0, 0)).unwrap();}
