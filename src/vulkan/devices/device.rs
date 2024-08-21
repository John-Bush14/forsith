use super::{c_void, c_char};

use crate::vulkan::{
    devices::physical_device::VkPhysicalDevice, VkBool32, VkResult
};


pub type VkDevice = u64;

pub type VkQueue = u64;


#[link(name = "vulkan")]
extern "C" {
    pub fn vkCreateDevice(
        physical_device: VkPhysicalDevice,
        device_create_info: *const VkDeviceCreateInfo, 
        _: *const c_void,
        device: *mut VkDevice
    ) -> VkResult;
    
    pub fn vkDestroyDevice(device: VkDevice, _: *const c_void);

    pub fn vkGetDeviceQueue(
        device: VkDevice,
        queue_family_index: u32,
        queue_index: u32,
        queue: *mut VkQueue
    );

    pub fn vkDeviceWaitIdle(device: VkDevice) -> VkResult;
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
pub struct VkPhysicalDeviceFeatures {
    pub buffer_front: [VkBool32;19],
    pub anisotropy: VkBool32,
    pub buffer_back: [VkBool32;35]
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
    pub enabled_features: *const VkPhysicalDeviceFeatures
}
