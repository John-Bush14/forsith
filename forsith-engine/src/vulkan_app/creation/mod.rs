use bindings::VkVersion;
use crate::DynError;
use super::VulkanApp;
use bindings::vk_version;


impl VulkanApp {
    fn new(app_name: &str, app_version: VkVersion) -> Result<Self, DynError> {todo!();}
}


#[cfg(test)]
#[test]
fn vulkan_app_creation_test() {VulkanApp::new("test", vk_version(0, 0, 0)).unwrap();}
