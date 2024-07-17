use crate::vulkan::{
    pipeline::{
        VkRect2D,
        VkOffset2D,
        VkViewport,
        VkShaderModule,
        VkShaderModuleCreateInfo,
        VkPipelineViewportStateCreateInfo,
        VkPipelineMultisampleStateCreateInfo,
        VkPipelineVertexInputStateCreateInfo,
        VkPipelineInputAssemblyStateCreateInfo,
        VkPipelineRasterizationStateCreateInfo,
        vkCreateShaderModule
    },
    devices::{
        device::{
            VkDevice
        }
    }
};

use std::io::{self, Read};


impl crate::engine::Engine { pub fn create_pipeline(&self) { unsafe {
    let vertex_shader = create_shader_module_from_file(&self.device, "src/engine/shaders/shader.vert.spv");
    
    let fragment_shader = create_shader_module_from_file(&self.device, "src/engine/shaders/shader.frag.spv");

    let vertex_input_state_create_info = VkPipelineVertexInputStateCreateInfo {
        s_type: 19,
        p_next: std::ptr::null(),
        flags: 0,
        vertex_binding_description_count: 0,
        vertex_binding_descriptions: std::ptr::null(),
        vertex_attribute_description_count: 0,
        vertex_attribute_descriptions: std::ptr::null()
    };

    let input_assembly_state_create_info = VkPipelineInputAssemblyStateCreateInfo {
        s_type: 20,
        p_next: std::ptr::null(),
        flags: 0,
        topology: 3,
        primitive_restart_enable: 1
    };

    let viewport = VkViewport {
        x: 0.0,
        y: 0.0,
        width: self.swapchain_extent.width as f32,
        height: self.swapchain_extent.height as f32,
        min_depth: 0.0,
        max_depth: 1.0
    };

    let viewports = [viewport];

    let scissor = VkRect2D {
        offset: VkOffset2D {x:0, y:0},
        extent: self.swapchain_extent.clone()
    };

    let scissors = [scissor];

    let viewport_state_create_info = VkPipelineViewportStateCreateInfo {
        s_type: 22,
        p_next: std::ptr::null(),
        flags: 0,
        viewport_count: viewports.len() as u32,
        viewports: viewports.as_ptr(),
        scissor_count: scissors.len() as u32,
        scissors: scissors.as_ptr()
    };

    let rasterization_state_create_info = VkPipelineRasterizationStateCreateInfo {
        s_type: 23,
        p_next: std::ptr::null(),
        flags: 0,
        depth_clamp_enable: 0,
        rasterizer_discard_enable: 0,
        polygon_mode: 0,
        cull_mode: 0x00000002,
        front_face: 1,
        depth_bias_enable: 0,
        depth_bias_constant_factor: 0.0,
        depth_bias_clamp: 0.0,
        depth_bias_slope_factor: 0.0,
        line_width: 1.0
    };

    let multisample_state_create_info = VkPipelineMultisampleStateCreateInfo {
        s_type: 24,
        p_next: std::ptr::null(),
        flags: 0,
        rasterization_samples: 0x00000001,
        sample_shading_enable: 0,
        min_sample_shading: 1.0,
        sample_mask: std::ptr::null(),
        alpha_to_coverage_enable: 0,
        alpha_to_one_enable: 0
    };
}}}

fn create_shader_module_from_file(device: &VkDevice, file: &str) -> VkShaderModule {
    let mut file = std::fs::File::open(file).expect("Nonexisting shader file");

    let mut raw_code = vec!();
    file.read_to_end(&mut raw_code);

    let code: Vec<u32> = raw_code.chunks_exact(4).map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])).collect();

    let create_info = VkShaderModuleCreateInfo {
        s_type: 16,
        p_next: std::ptr::null(),
        flags: 0,
        code_size: code.len(),
        code: code.as_ptr()
    };
    
    let mut shader_module: VkShaderModule = unsafe{std::mem::zeroed()};

    unsafe { vkCreateShaderModule(
        *device,
        &create_info as *const VkShaderModuleCreateInfo,
        std::ptr::null(),
        &mut shader_module as *mut VkShaderModule
    )};
    
    return shader_module;
}
