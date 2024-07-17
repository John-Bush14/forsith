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


pub type VkShaderModule = u64;


#[repr(C)]
pub struct VkShaderModuleCreateInfo {
    pub s_type: VkStructureType,
    pub p_next: *const c_void,
    pub flags: u32,
    pub code_size: usize,
    pub code: *const u32
}


#[link(name = "vulkan")]
extern "C" {
    pub fn vkCreateShaderModule(
        device: VkDevice,
        create_info: *const VkShaderModuleCreateInfo,
        _: *const c_void,
        shader_module: *mut VkShaderModule
    ) -> VkResult;
}
