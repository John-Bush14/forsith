use crate::vulkan::{
    devices::{
        device::{
            VkDevice
        }
    },
    VkResult,
    VkStructureType
};

use std::ffi::{
    c_void
};


pub type VkCommandPool = u64;


#[repr(C)]
pub struct VkCommandPoolCreateInfo {
    pub s_type: VkStructureType, //39
    pub p_next: *const c_void,
    pub flags: u32,
    pub queue_family_index: u32
}


#[link(name = "vulkan")]
extern "C" { 
    pub fn vkCreateCommandPool(
        device: VkDevice,
        create_info: *const VkCommandPoolCreateInfo,
        _: *const c_void,
        command_pool: *mut VkCommandPool
    ) -> VkResult;

    pub fn vkDestroyCommandPool(
        device: VkDevice,
        command_pool: VkCommandPool,
        _: *const c_void
    );
}
