use bindings::instance::vk_destroy_instance;

use super::VulkanApp;


impl Drop for VulkanApp {
    fn drop(&mut self) {
        self.transient_command_pool.destroy();

        self.general_device.destroy().unwrap();

        vk_destroy_instance(self.instance, std::ptr::null());
    }
}
