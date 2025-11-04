use bindings::instance::vk_destroy_instance;

use super::VulkanApp;


impl Drop for VulkanApp {
    fn drop(&mut self) {
        vk_destroy_instance(self.instance, std::ptr::null());
    }
}
