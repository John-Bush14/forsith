use super::{CString, c_void, c_char};

pub type VkDevice = u64;

pub const VK_CREATE_DEVICE_CSTRING: fn() -> CString = || CString::new("vkCreateDevice").unwrap();

pub const VK_GET_PHYSICAL_DEVICE_QUEUE_FAMILY_PROPERTIES_CSTRING: fn() -> CString = 
    || CString::new("vkGetPhysicalDeviceQueueFamilyProperties").unwrap();

#[repr(C)]
pub struct VkQueueFamilyProperties {
    pub flags: u32,
    pub count: u32,
    timestamp_valid_bits: u32,
    min_image_transfer_granulatity: c_void
}

#[repr(C)]
pub struct VkDeviceQueueCreateInfo {
    pub s_type: super::VkStructureType,
    pub p_next: *const c_void,
    pub flags: u32,
    pub queue_family_index: u32,
    pub queue_count: u32,
    pub queue_priorities: *const f32
}

#[repr(C)]
pub struct VkDeviceCreateInfo {
    pub s_type: super::VkStructureType,
    pub p_next: *const c_void,
    pub flags: u32,
    pub queue_create_info_count: u32,
    pub queue_create_infos: *const VkDeviceQueueCreateInfo,
    pub enabled_layer_count: u32,
    pub enabled_layer_names: *const *const c_char,
    pub enabled_extension_count: u32,
    pub enabled_extension_names: *const *const c_char,
    pub enabled_features: *const c_void
}
