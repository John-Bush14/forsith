use bindings::VkVersion;
use crate::DynError;
use super::VulkanApp;


mod instance;


impl VulkanApp {
    pub fn new(name: &str, version: VkVersion) -> Result<Self, DynError> {
        let instance = instance::create_instance(name, version)?;


        return Ok(VulkanApp { instance });
    }
}
