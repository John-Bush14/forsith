use crate::vulkan::{
    pipeline::{
        VkShaderModule,
        VkShaderModuleCreateInfo,
        VkPipelineVertexInputStateCreateInfo,
        VkPipelineInputAssemblyStateCreateInfo,
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
