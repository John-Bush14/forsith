use bindings::{command_pool::{VkCommandPoolCreateFlagBits, VkCommandPoolCreateFlags}, VkVersion};
use crate::{DynError};
use super::VulkanApp;


pub(crate) mod instance;


#[derive(Default)]
pub struct VulkanAppLimits {
    pub renderers: u8,
    pub processing_queues: u8
}

impl VulkanAppLimits {
    pub fn get_renderers(&self) -> u8 {return self.renderers;}
    pub fn get_processing_queues(&self) -> u8 {return self.processing_queues;}
}


#[allow(dead_code)]
impl VulkanApp {
    fn new(app_name: &str, app_version: VkVersion, limits: VulkanAppLimits) -> Result<Self, DynError> {
        let instance = instance::create_instance(app_name, app_version)?;

        return Ok(Self {
            instance,
        });
    }
}


#[cfg(test)]
#[test]
fn vulkan_app_creation_test() {
    use bindings::vk_version;

    VulkanApp::new("test", vk_version(0, 0, 0), VulkanAppLimits {
        renderers: 1,
        processing_queues: 1,
    }).unwrap();
}
