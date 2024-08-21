use crate::vulkan::{
    devices::device::VkDevice,
    VkResult,
    VkStructureType
};

use std::ffi::c_void;

use super::{commands::command_buffer::VkCommandBuffer, vertex::{VkBuffer, VkDeviceMemory, VkMemoryRequirements}, VkBool32};


pub type VkImageView = u64;

pub type VkImage = u64;

pub type VkSampler = u64;


#[derive(Clone)]
pub struct Texture {
    pub image: VkImage, 
    pub memory: VkDeviceMemory,
    pub image_view: VkImageView, 
    pub sampler: VkSampler
}

#[repr(C)]
pub struct VkDescriptorImageInfo {
    pub sampler: VkSampler,
    pub image_view: VkImageView,
    pub image_layout: u32
}

#[repr(C)]
pub struct VkBufferImageCopy {
    pub buffer_offset: u64,
    pub buffer_row_length: u32,
    pub buffer_image_height: u32,
    pub image_subresource: VkImageSubresourceLayers,
    pub image_offset: VkOffset3D,
    pub image_extent: VkExtent3D
}

#[repr(C)]
pub struct VkOffset3D {
   pub x: i32,
   pub y: i32,
   pub z: i32
}

#[repr(C)]
pub struct VkImageSubresourceLayers {
    pub aspect_mask: u32,
    pub mip_level: u32,
    pub base_array_layer: u32,
    pub layer_count: u32
}

#[repr(C)]
pub struct VkImageMemoryBarrier {
    pub s_type: VkStructureType,
    pub p_next: *const c_void,
    pub src_access_mask: u32,
    pub dst_access_mask: u32,
    pub old_layout: u32,
    pub new_layout: u32,
    pub src_queue_family_index: u32,
    pub dst_queue_family_index: u32,
    pub image: VkImage,
    pub subresource_range: VkImageSubresourceRange
}

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
pub struct VkSamplerCreateInfo {
    pub s_type: VkStructureType,
    pub p_next: *const c_void,
    pub flags: u32,
    pub mag_filter: u32,
    pub min_filter: u32,
    pub mipmap_mode: u32,
    pub address_mode_u: u32,
    pub addres_mode_v: u32,
    pub address_mode_w: u32,
    pub mip_lod_bias: f32,
    pub anisotropy_enable: VkBool32,
    pub max_anisotropy: f32,
    pub compare_enable: VkBool32,
    pub compare_op: u32,
    pub min_lod: f32,
    pub max_lod: f32,
    pub border_color: u32,
    pub unnormalized_coordinates: VkBool32
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

    pub fn vkCmdPipelineBarrier(
        command_buffer: VkCommandBuffer,
        src_stage_mask: u32,
        dst_stage_mask: u32,
        dependency_flags: u32,
        memory_barrier_count: u32,
        memory_barriers: *const c_void,
        buffer_memory_barrier_count: u32,
        buffer_memory_barriers: *const c_void,
        image_memory_barrier_count: u32,
        image_memory_barriers: *const VkImageMemoryBarrier
    );

    pub fn vkDestroyImage(
        device: VkDevice,
        image: VkImage,
        _: *const c_void
    );

    pub fn vkCmdCopyBufferToImage(
        command_buffer: VkCommandBuffer,
        src_buffer: VkBuffer,
        dst_image: VkImage,
        dst_image_layout: u32,
        region_count: u32,
        regions: *const VkBufferImageCopy
    );
    
    pub fn vkCreateSampler(
        device: VkDevice,
        create_info: *const VkSamplerCreateInfo,
        _: *const c_void,
        sampler: *mut VkSampler
    ) -> VkResult;

    pub fn vkDestroySampler(
        device: VkDevice,
        sampler: VkSampler,
        _: *const c_void
    );
}
