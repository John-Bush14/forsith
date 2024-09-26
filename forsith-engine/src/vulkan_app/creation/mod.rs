use bindings::VkVersion;
use crate::DynError;
use super::VulkanApp;
use bindings::vk_version;


mod instance;


#[allow(dead_code)]
pub const API_VERSION: fn() -> VkVersion = || vk_version(1, 0, 0);


#[allow(dead_code)]
impl VulkanApp {
    fn new(app_name: &str, app_version: VkVersion) -> Result<Self, DynError> {
        let instance = instance::create_instance(app_name, app_version);

        todo!();
    }
}


#[cfg(test)]
#[test]
fn vulkan_app_creation_test() {VulkanApp::new("test", vk_version(0, 0, 0)).unwrap();}
