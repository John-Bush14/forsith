use crate::vulkan::{
    instance::{
        vkDestroyInstance
    },
    devices::{
        device::{
            vkDestroyDevice
        }
    },
};

impl Drop for super::Engine {
    fn drop(&mut self) {
        unsafe {
            self.window.commit_suicide();

            vkDestroyDevice(self.device, std::ptr::null());

            vkDestroyInstance(self.instance, std::ptr::null());
        };
    }
}
