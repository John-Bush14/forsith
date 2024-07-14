use crate::vulkan::{
    instance::{
        vkDestroyInstance
    }
};

impl Drop for super::Engine {
    fn drop(&mut self) {
        unsafe {
            vkDestroyInstance(self.instance, std::ptr::null());
        };
    }
}
