use crate::vulkan::{
    devices::{
        device::{
            VkDevice
        }
    },
    swapchain::{
        VkImage
    },
    VkResult,
    VkStructureType
};

use std::ffi::{
    c_void
};


pub type VkImageView = u64;


#[repr(C)]
pub struct VkComponentMapping {
    pub r: u32,
    pub g: u32,
    pub b: u32,
    pub a: u32
}

#[repr(C)]
pub struct VkImageSubresourceRange {
    pub aspect_mask: u32,
    pub base_mip_level: u32,
    pub level_count: u32,
    pub base_array_layer: u32,
    pub layer_count: u32
}

#[repr(C)]
pub struct VkImageViewCreateInfo {
    pub s_type: VkStructureType,
    pub p_next: *const c_void,
    pub flags: u32,
    pub image: VkImage,
    pub view_type: u32,
    pub format: u32,
    pub components: VkComponentMapping,
    pub subresource_range: VkImageSubresourceRange
}


#[link(name = "vulkan")]
extern "C" {
    pub fn vkCreateImageView(
        device: VkDevice,
        create_info: *const VkImageViewCreateInfo,
        _: *const c_void,
        image_view: *mut VkImageView
    ) -> VkResult;

    pub fn vkDestroyImageView(
        device: VkDevice,
        image_view: VkImageView,
        _: *const c_void
    );
}
