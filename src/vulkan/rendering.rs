use crate::vulkan::{
    devices::device::{
        VkDevice,
        VkQueue
    },
    commands::command_buffer::VkCommandBuffer,
    swapchain::VkSwapchainKHR,
    VkBool32,
    VkResult,
    VkStructureType
};

use std::ffi::c_void;


pub type VkSemaphore = u64;

pub type VkFence = u64;


pub const MAX_FRAMES_IN_FLIGHT: u32 = 2;


#[repr(C)]
pub struct VkPresentInfoKHR {
    pub s_type: VkStructureType,
    pub p_next: *const c_void,
    pub wait_semaphore_count: u32,
    pub wait_semaphores: *const VkSemaphore,
    pub swapchain_count: u32,
    pub swapchains: *const VkSwapchainKHR,
    pub image_indices: *const u32,
    pub results: *mut VkResult
}

#[repr(C)]
pub struct VkSubmitInfo {
    pub s_type: VkStructureType,
    pub p_next: *const c_void,
    pub wait_sephamore_count: u32,
    pub wait_sephamores: *const VkSemaphore,
    pub wait_dst_stage_mask: *const u32,
    pub command_buffer_count: u32,
    pub command_buffers: *const VkCommandBuffer,
    pub signal_sephamore_count: u32,
    pub signal_sephamores: *const VkSemaphore
}

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

    pub fn vkWaitForFences(
        device: VkDevice,
        fence_count: u32,
        fences: *const VkFence,
        wait_all: VkBool32,
        timeout: u64
    ) -> VkResult;

    pub fn vkResetFences(
        device: VkDevice,
        fence_count: u32,
        fences: *const VkFence
    ) -> VkResult; 

    pub fn vkQueueSubmit(
        queue: VkQueue,
        submit_count: u32,
        submits: *const VkSubmitInfo,
        fence: VkFence
    ) -> VkResult;

    pub fn vkQueuePresentKHR(queue: VkQueue, present_info: *const VkPresentInfoKHR) -> VkResult;

    pub fn vkQueueWaitIdle(queue: VkQueue) -> VkResult;
}
