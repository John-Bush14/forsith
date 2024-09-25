use crate::{define_vk_bitmask, define_vk_struct, structure_type::VkStructureType, vk_result::VkResult, VkAllocationCallbacks, VkHandle, VkVersion};
use std::ffi::c_char;


pub type VkInstance = VkHandle;


define_vk_bitmask!(VkInstanceCreateFlags(VkInstanceCreateFlagBits) {
    VK_INSTANCE_CREATE_ENUMERATE_PORTABILITY_BIT_KHR = 0x00000001,
});


define_vk_struct!(pub VkApplicationInfo(VkStructureType::VkStructureTypeApplicationInfo) {
    application_name: *const c_char,
    application_version: VkVersion,
    engine_name: *const c_char,
    engine_version: VkVersion,
    api_version: VkVersion
});

define_vk_struct!(pub VkInstanceCreateInfo(VkStructureType::VkStructureTypeInstanceCreateInfo) {
    flags: VkInstanceCreateFlags,
    application_info: *const VkApplicationInfo,
    enabled_layer_count: u32,
    enabled_layer_names: *const *const c_char,
    enabled_extension_count: u32,
    enabled_extensions: *const *const c_char
});


#[link(name = "vulkan")]
extern "C" {
    pub fn vkCreateInstance(
        create_info: *const VkInstanceCreateInfo,
        allocator: *const VkAllocationCallbacks,
        instance: *mut VkInstance
    ) -> VkResult;
}


#[cfg(test)]
#[test]
fn test_vulkan_link() {unsafe {vkCreateInstance(std::ptr::null(), std::ptr::null(), std::ptr::null_mut());}}
