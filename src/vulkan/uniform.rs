use crate::vulkan::{
    devices::device::VkDevice,
    vertex::VkBuffer,
    VkResult,
    VkStructureType
};

use std::ffi::c_void;


pub type VkDescriptorSetLayout = u64;

pub type VkDescriptorPool = u64;

pub type VkDescriptorSet = u64;


#[link(name = "vulkan")]
extern "C" { 
    pub fn vkCreateDescriptorSetLayout(
        device: VkDevice,
        create_info: *const VkDescriptorSetLayoutCreateInfo,
        _: *const c_void,
        descriptor_set_layout: *mut VkDescriptorSetLayout
    ) -> VkResult;

    pub fn vkDestroyDescriptorSetLayout(
        device: VkDevice,
        descriptor_set_layout: VkDescriptorSetLayout,
        _: *const c_void
    );

    pub fn vkCreateDescriptorPool(
        device: VkDevice,
        create_device: *const VkDescriptorPoolCreateInfo,
        _: *const c_void,
        descriptor_pool: *mut VkDescriptorPool
    ) -> VkResult;

    pub fn vkUpdateDescriptorSets(
        device: VkDevice,
        descriptor_write_count: u32,
        descriptor_writes: *const VkWriteDescriptorSet,
        descriptor_copy_count: u32,
        descriptor_copies: *const VkCopyDescriptorSet
    );

    pub fn vkAllocateDescriptorSets(
        device: VkDevice,
        allocate_info: *const VkDescriptorSetAllocateInfo,
        descriptor_sets: *mut VkDescriptorSet
    ) -> VkResult;

    pub fn vkDestroyDescriptorPool(
        device: VkDevice,
        descriptor_pool: VkDescriptorPool,
        _: *const c_void
    );
}


#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct UniformBufferObject {
    pub model: [[f32;4];4],
    pub view: [[f32;4];4],
    pub proj: [[f32;4];4],
}

#[repr(C)]
pub struct VkDescriptorSetAllocateInfo {
    pub s_type: VkStructureType,
    pub p_next: *const c_void,
    pub descriptor_pool: VkDescriptorPool,
    pub descriptor_set_count: u32,
    pub set_layouts: *const VkDescriptorSetLayout
}

#[repr(C)]
pub struct VkWriteDescriptorSet {
    pub s_type: VkStructureType,
    pub p_next: *const c_void,
    pub dst_set: VkDescriptorSet,
    pub dst_binding: u32,
    pub dst_array_element: u32,
    pub descriptor_count: u32,
    pub descriptor_type: u32,
    pub image_info: *const c_void,
    pub buffer_info: *const VkDescriptorBufferInfo,
    pub texel_buffer_view: *const c_void
}

#[repr(C)]
pub struct VkDescriptorBufferInfo {
    pub buffer: VkBuffer,
    pub offset: u64,
    pub range: u64
}

#[repr(C)]
pub struct VkCopyDescriptorSet {
    pub s_type: VkStructureType,
    pub p_next: *const c_void,
    pub src_set: VkDescriptorSet,
    pub src_binding: u32,
    pub src_array_element: u32,
    pub dst_set: VkDescriptorSet,
    pub dst_binding: u32,
    pub dst_array_element: u32,
    pub descriptor_count: u32
}

#[repr(C)]
pub struct VkDescriptorPoolCreateInfo {
    pub s_type: VkStructureType,
    pub p_next: *const c_void,
    pub flags: u32,
    pub max_sets: u32,
    pub pool_size_count: u32,
    pub pool_sizes: *const VkDescriptorPoolSize
}

#[repr(C)]
pub struct VkDescriptorPoolSize {
    pub type_: u32,
    pub descriptor_count: u32
}

#[repr(C)]
pub struct VkDescriptorSetLayoutBinding {
    pub binding: u32,
    pub descriptor_type: u32,
    pub descriptor_count: u32,
    pub stage_flags: u32,
    pub immutable_samplers: *const c_void
}

#[repr(C)]
pub struct VkDescriptorSetLayoutCreateInfo {
    pub s_type: VkStructureType,
    pub p_next: *const c_void,
    pub flags: u32,
    pub binding_count: u32,
    pub bindings: *const VkDescriptorSetLayoutBinding
}
