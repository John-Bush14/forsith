use crate::vulkan::{
    devices::device::VkDevice,
    VkResult,
    VkStructureType
};

use std::ffi::c_void;

use super::vertex::{VkDeviceMemory, VkMemoryRequirements};


pub type VkImageView = u64;

pub type VkImage = u64;


#[repr(C)]
pub struct VkExtent3D {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) depth: u32,
}

#[repr(C)]
pub struct VkImageCreateInfo {
    pub(crate) s_type: VkStructureType,
    pub(crate) p_next: *const c_void,
    pub(crate) flags: u32,
    pub(crate) image_type: u32,
    pub(crate) format: u32,
    pub(crate) extent: VkExtent3D,
    pub(crate) miplevels: u32,
    pub(crate) array_layers: u32,
    pub(crate) samples: u32,
    pub(crate) tiling: u32,
    pub(crate) usage: u32,
    pub(crate) sharing_mode: u32,
    pub(crate) queue_family_index_count: u32,
    pub(crate) queue_family_indices: *const u32,
    pub(crate) initial_layout: u32
}

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

    pub fn vkCreateImage(
        device: VkDevice,
        create_info: *const VkImageCreateInfo,
        _: *const c_void,
        image_view: *mut VkImage
    ) -> VkResult;

    pub fn vkGetImageMemoryRequirements(
        device: VkDevice,
        image: VkImage,
        memory_requirements: *mut VkMemoryRequirements
    );

    pub fn vkBindImageMemory(
        device: VkDevice,
        image: VkImage,
        memory: VkDeviceMemory,
        offset: u64
    ) -> VkResult;
}
