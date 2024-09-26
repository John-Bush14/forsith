use bindings::VkVersion;
use crate::DynError;
use super::VulkanApp;


mod instance;


#[allow(dead_code)]
impl VulkanApp {
    fn new(app_name: &str, app_version: VkVersion) -> Result<Self, DynError> {
        let instance = instance::create_instance(app_name, app_version)?;


        return Ok(Self {
            instance,
            device: 0,
            physical_device: 0,
            transient_command_pool: 0,
            graphics_queue_family: 0,
            graphics_queue: 0,
        });
    }
}


#[cfg(test)]
#[test]
fn vulkan_app_creation_test() {
    use bindings::vk_version;

    VulkanApp::new("test", vk_version(0, 0, 0)).unwrap();
}
