use crate::vulkan::{
    devices::{
        device::{
            VkDevice
        }
    },
    swapchain::{
        VkExtent2D
    },
    VkBool32,
    VkResult,
    VkStructureType
};

use std::ffi::{
    c_void
};


pub type VkShaderModule = u64;


#[repr(C)]
pub struct VkPipelineRasterizationStateCreateInfo {
    pub s_type: VkStructureType,
    pub p_next: *const c_void,
    pub flags: u32,
    pub depth_clamp_enable: VkBool32,
    pub rasterizer_discard_enable: VkBool32,
    pub polygon_mode: u32,
    pub cull_mode: u32,
    pub front_face: u32,
    pub depth_bias_enable: VkBool32,
    pub depth_bias_constant_factor: f32,
    pub depth_bias_clamp: f32,
    pub depth_bias_slope_factor: f32,
    pub line_width: f32
}

#[repr(C)]
pub struct VkPipelineMultisampleStateCreateInfo {
    pub s_type: VkStructureType,
    pub p_next: *const c_void,
    pub flags: u32,
    pub rasterization_samples: u32,
    pub sample_shading_enable: VkBool32,
    pub min_sample_shading: f32,
    pub sample_mask: *const c_void, // temp
    pub alpha_to_coverage_enable: VkBool32,
    pub alpha_to_one_enable: VkBool32
}

#[repr(C)]
pub struct VkPipelineVertexInputStateCreateInfo {
    pub s_type: VkStructureType,
    pub p_next: *const c_void,
    pub flags: u32,
    pub vertex_binding_description_count: u32,
    pub vertex_binding_descriptions: *const c_void,
    pub vertex_attribute_description_count: u32,
    pub vertex_attribute_descriptions: *const c_void,
}

#[repr(C)]
pub struct VkViewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub min_depth: f32,
    pub max_depth: f32
}

#[repr(C)]
pub struct VkOffset2D {
    pub x: i32,
    pub y: i32
}

#[repr(C)]
pub struct VkRect2D {
    pub offset: VkOffset2D,
    pub extent: VkExtent2D
}

#[repr(C)]
pub struct VkPipelineViewportStateCreateInfo {
    pub s_type: VkStructureType,
    pub p_next: *const c_void,
    pub flags: u32,
    pub viewport_count: u32,
    pub viewports: *const VkViewport,
    pub scissor_count: u32,
    pub scissors: *const VkRect2D
}

#[repr(C)]
pub struct VkPipelineInputAssemblyStateCreateInfo {
    pub s_type: VkStructureType,
    pub p_next: *const c_void,
    pub flags: u32,
    pub topology: u32,
    pub primitive_restart_enable: VkBool32
}

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
