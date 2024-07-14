use super::{CString, c_void, c_char};

use crate::vulkan::{
    devices::{
        physical_device::{
            VkPhysicalDevice
        }
    },
    VkResult
};


pub type VkDevice = u64;


#[link(name = "vulkan")]
extern "C" {
    pub fn vkCreateDevice(
        physical_device: VkPhysicalDevice,
        device_create_info: *const VkDeviceCreateInfo, 
        _: *const c_void,
        device: *mut VkDevice
    ) -> VkResult;
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
