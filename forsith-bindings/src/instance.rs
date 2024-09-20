use crate::{vk_create_info, VkAllocationCallbacks, VkBitmask, VkHandle, result::VkResult, VkVersion};
use std::ffi::c_char;


pub type VkInstance = VkHandle;


vk_create_info!(pub VkApplicationInfo {
    p_application_name: *const c_char,
    application_version: VkVersion,
    p_engine_name: *const c_char,
    engine_version: VkVersion,
    api_version: VkVersion
});

vk_create_info!(pub VkInstanceCreateInfo {
    flags: VkBitmask,
    p_application_info: *const VkApplicationInfo,
    enabled_layer_count: u32,
    pp_enabled_layer_names: *const *const c_char,
    enabled_extension_count: u32,
    pp_enabled_extension_names: *const *const c_char
});


#[link(name = "vulkan")]
extern "C" {
    pub fn vkCreateInstance(
        p_create_info: *const VkInstanceCreateInfo,
        p_allocator: VkAllocationCallbacks,
        mp_instance: *mut VkInstance
    ) -> VkResult;
}
