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


pub type VkSemaphore = u64;

pub type VkFence = u64;


pub const MAX_FRAMES_IN_FLIGHT: u32 = 2;


#[repr(C)]
pub struct VkSemaphoreCreateInfo {
    pub s_type: VkStructureType,
    pub p_next: *const c_void,
    pub flags: u32
}

#[repr(C)]
pub struct VkFenceCreateInfo {
    pub s_type: VkStructureType,
    pub p_next: *const c_void,
    pub flags: u32
}


#[link(name = "vulkan")]
extern "C" {
    pub fn vkCreateSemaphore(
        device: VkDevice,
        create_info: *const VkSemaphoreCreateInfo,
        _: *const c_void,
        semaphore: *mut VkSemaphore
    ) -> VkResult;

    pub fn vkCreateFence(
        device: VkDevice,
        create_info: *const VkFenceCreateInfo,
        _: *const c_void,
        fence: *mut VkFence
    ) -> VkResult;

    pub fn vkDestroyFence(
        device: VkDevice,
        fence: VkFence,
        _: *const c_void
    );

    pub fn vkDestroySemaphore(
        device: VkDevice,
        semaphore: VkSemaphore,
        _: *const c_void
    );
}
