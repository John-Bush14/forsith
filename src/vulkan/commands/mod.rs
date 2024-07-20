use crate::vulkan::{
    commands::{
        command_buffer::{
            VkCommandBuffer,
        },
    },
    pipeline::{
        VkRect2D,
        VkPipeline,
        VkRenderPass,
        VkFramebuffer
    },
    VkStructureType,
};

use std::ffi::{
    c_void
};


#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct VkClearDepthStencilValue {
    pub depth: f32,
    pub stencil: u32
}

#[derive(Clone, Copy)]
#[repr(C)]
pub union VkClearColorValue {
    pub float32: [f32;4],
    pub int32: [i32;4],
    pub uint32: [u32;4]
}

#[repr(C)]
pub union VkClearValue {
    pub color: VkClearColorValue,
    pub depth_stencil: VkClearDepthStencilValue
}

#[repr(C)]
pub struct VkRenderPassBeginInfo {
    pub s_type: VkStructureType,
    pub p_next: *const c_void,
    pub render_pass: VkRenderPass,
    pub framebuffer: VkFramebuffer,
    pub render_area: VkRect2D,
    pub clear_value_count: u32,
    pub clear_values: *const VkClearValue
}


#[link(name = "vulkan")]
extern "C" { 
    pub fn vkCmdDraw(
        command_buffer: VkCommandBuffer,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32
    );

    pub fn vkCmdBeginRenderPass(
        command_buffer: VkCommandBuffer,
        begin_info: *const VkRenderPassBeginInfo,
        contents: u32
    );

    pub fn vkCmdBindPipeline(
        command_buffer: VkCommandBuffer,
        pipeline_bind_point: u32,
        pipeline: VkPipeline
    );

    pub fn vkCmdEndRenderPass(command_buffer: VkCommandBuffer);
}


pub mod command_buffer;

pub mod command_pool;
