use bindings::{command_pool::{VkCommandPoolCreateFlagBits, VkCommandPoolCreateFlags}, VkVersion};
use crate::{command_pool::CommandPool, vulkan_app::device::create_device, DynError};
use super::VulkanApp;


pub(crate) mod instance;


#[derive(Default)]
pub struct VulkanAppLimits {
    renderers: u8,
    processing_queues: u8
}

impl VulkanAppLimits {
    pub fn get_renderers(&self) -> u8 {return self.renderers;}
    pub fn get_processing_queues(&self) -> u8 {return self.processing_queues;}
}


#[allow(dead_code)]
impl VulkanApp {
    fn new(app_name: &str, app_version: VkVersion, limits: VulkanAppLimits) -> Result<Self, DynError> {
        let instance = instance::create_instance(app_name, app_version)?;

        let general_device = create_device(instance, &limits)?;


        let transient_command_pool = CommandPool::new(
            &general_device,
            VkCommandPoolCreateFlags(VkCommandPoolCreateFlagBits::VkCommandPoolCreateTransientBit as _),
            general_device.get_processing_queue(0).family()
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

    VulkanApp::new("test", vk_version(0, 0, 0), VulkanAppLimits {
        renderers: 0,
        processing_queues: 0,
    }).unwrap();
}
