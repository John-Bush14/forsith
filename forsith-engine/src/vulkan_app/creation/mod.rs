use bindings::VkVersion;
use super::VulkanApp;


mod instance;


impl VulkanApp {
    pub fn new(name: &str, version: VkVersion) -> Self {
        let instance = instance::create_instance(name, version);


        return VulkanApp { instance };
    }
}
