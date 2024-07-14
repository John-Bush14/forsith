use super::{CString, c_void, c_char};

pub const VK_CREATE_INSTANCE_CSTRING: fn() -> CString = || CString::new("vkCreateInstance").unwrap();

use super::VkResult;

pub type VkInstance = u64;

#[link(name = "vulkan")]
extern "C" { 
    pub fn vkCreateInstance(
        instance_create_info: *const VkInstanceCreateInfo, _: *const c_void, instance: *mut VkInstance
    ) -> VkResult;

    pub fn vkDestroyInstance(
        instance: VkInstance, _: *const c_void
    ) -> VkResult;

    pub fn vkEnumerateInstanceExtensionProperties(
        layer_name: *const c_char,
        extension_property_count: *mut u32,
        extension_properties: *mut VkExtensionProperties
    ) -> VkResult;
}


#[repr(C)]
pub struct VkExtensionProperties {
    pub extension_name: [c_char; 256],
    pub spec_version: u32
}

#[repr(C)]
pub struct VkApplicationInfo {
    pub s_type: super::VkStructureType,
    pub p_next: *const c_void,
    pub p_application_name: *const c_char,
    pub application_version: u32,
    pub p_engine_name: *const c_char,
    pub engine_version: u32,
    pub api_version: u32,
}

#[repr(C)]
pub struct VkInstanceCreateInfo {
    pub s_type: super::VkStructureType,
    pub p_next: *const c_void,
    pub flags: u32,
    pub p_application_info: *const VkApplicationInfo,
    pub enabled_layer_count: u32,
    pub pp_enabled_layer_names: *const *const c_char,
    pub enabled_extension_count: u32,
    pub pp_enabled_extension_names: *const *const c_char,
}
