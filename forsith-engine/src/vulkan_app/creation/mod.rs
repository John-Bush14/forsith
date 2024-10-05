use bindings::{physical_device::{VkQueueFamilyProperties, VkQueueFlagBits}, Bitmask, VkVersion};
use crate::{device::create_device, DynError};
use super::VulkanApp;


pub(crate) mod instance;


#[allow(dead_code)]
impl VulkanApp {
    fn new(app_name: &str, app_version: VkVersion) -> Result<Self, DynError> {
        let instance = instance::create_instance(app_name, app_version)?;

        let general_device = create_device(instance, vec![|_physical_device, queue_family_props: VkQueueFamilyProperties| {
            return queue_family_props.queue_flags.contains(VkQueueFlagBits::VkQueueGraphicsBit)
        }])?;

        return Ok(Self {
            instance,
            general_device,
            transient_command_pool: 0,
        });
    }
}


#[cfg(test)]
#[test]
fn vulkan_app_creation_test() {
    use bindings::vk_version;

    VulkanApp::new("test", vk_version(0, 0, 0)).unwrap();
}
