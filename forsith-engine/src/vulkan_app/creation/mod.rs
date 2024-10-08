use bindings::{command_pool::{VkCommandPoolCreateFlagBits, VkCommandPoolCreateFlags}, VkVersion};
use crate::{command_pool::CommandPool, device::create_device, DynError};
use super::VulkanApp;


pub(crate) mod instance;


pub struct VulkanAppLimits {
    renderers: u8
}


#[allow(dead_code)]
impl VulkanApp {
    fn new(app_name: &str, app_version: VkVersion, limits: VulkanAppLimits) -> Result<Self, DynError> {
        let instance = instance::create_instance(app_name, app_version)?;

        let general_device = create_device(instance, limits.renderers)?;


        let transient_command_pool = CommandPool::new(
            &general_device,
            VkCommandPoolCreateFlags(VkCommandPoolCreateFlagBits::VkCommandPoolCreateTransientBit as _),
            general_device.get_queue(0).family()
        )?;


        return Ok(Self {
            instance,
            general_device,
            transient_command_pool,
        });
    }
}


#[cfg(test)]
#[test]
fn vulkan_app_creation_test() {
    use bindings::vk_version;

    VulkanApp::new("test", vk_version(0, 0, 0), 1).unwrap();
}
