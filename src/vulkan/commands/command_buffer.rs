use crate::vulkan::{
    devices::device::VkDevice,
    commands::command_pool::VkCommandPool,
    VkResult,
    VkStructureType
};

use std::ffi::c_void;


pub type VkCommandBuffer = u64;


#[repr(C)]
pub struct VkCommandBufferAllocateInfo {
    pub s_type: VkStructureType,
    pub p_next: *const c_void,
    pub command_pool: VkCommandPool,
    pub level: u32,
    pub command_buffer_count: u32
}

#[repr(C)]
pub struct VkCommandBufferBeginInfo {
    pub s_type: VkStructureType,
    pub p_next: *const c_void,
    pub flags: u32,
    pub inheritance_info: *const c_void
}


#[link(name = "vulkan")]
extern "C" {
    pub fn vkAllocateCommandBuffers(
        device: VkDevice,
        create_info: *const VkCommandBufferAllocateInfo,
        command_buffers: *mut VkCommandBuffer
    ) -> VkResult;

    pub fn vkBeginCommandBuffer(
        command_buffer: VkCommandBuffer,
        begin_info: *const VkCommandBufferBeginInfo
    ) -> VkResult;

    pub fn vkEndCommandBuffer(
        command_buffer: VkCommandBuffer
    ) -> VkResult;

    pub fn vkFreeCommandBuffers(
        device: VkDevice,
        command_pool: VkCommandPool,
        command_buffer_count: u32,
        command_buffers: *const VkCommandBuffer
    );
}
