use std::ffi::{c_void, c_char};

use crate::vulkan::{devices::physical_device::{self, VkPhysicalDevice}, VkBool32};


#[link(name = "wayland-client")]
extern "C" {
    pub fn wl_display_connect(name: *const c_char) -> *const c_void;
    pub fn wl_display_disconnect(display: *const c_void);
}

#[link(name = "vulkan")]
extern "C" {
    pub fn vkGetPhysicalDeviceWaylandPresentationSupportKHR(
        physical_device: VkPhysicalDevice,
        queue_family_index: u32,
        display: *const c_void
    ) -> VkBool32;
}


pub struct WayWindow {
    pub display: *const c_void
}
