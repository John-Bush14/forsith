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


pub type VkDescriptorSetLayout = u64;


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
}


#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct UniformBufferObject {
    pub model: [[f32;4];4],
    pub view: [[f32;4];4],
    pub proj: [[f32;4];4],
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
