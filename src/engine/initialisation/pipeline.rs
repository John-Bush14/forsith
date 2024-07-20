use crate::vulkan::{
    pipeline::{
        VkRect2D,
        VkOffset2D,
        VkViewport,
        VkRenderPass,
        VkShaderModule,
        VkSubpassDescription,
        VkAttachmentReference,
        VkRenderPassCreateInfo,
        VkAttachmentDescription,
        VkShaderModuleCreateInfo,
        VkPipelineLayoutCreateInfo,
        VkGraphicsPipelineCreateInfo,
        VkPipelineShaderStageCreateInfo,
        VkPipelineViewportStateCreateInfo,
        VkPipelineColorBlendAttachmentState,
        VkPipelineColorBlendStateCreateInfo,
        VkPipelineMultisampleStateCreateInfo,
        VkPipelineVertexInputStateCreateInfo,
        VkPipelineDepthStencilStateCreateInfo,
        VkPipelineTessellationStateCreateInfo,
        VkPipelineInputAssemblyStateCreateInfo,
        VkPipelineRasterizationStateCreateInfo,
        vkCreateRenderPass,
        vkCreateShaderModule,
        vkCreatePipelineLayout,
        vkCreateGraphicsPipelines
    },
    devices::{
        device::{
            VkDevice
        }
    }
};

use std::io::{self, Read};

use std::ffi::{
    CString
};


impl crate::engine::Engine { pub fn create_pipeline(&mut self) { unsafe {
    let entry_point_name = CString::new("main").unwrap();

    let vertex_shader = create_shader_module_from_file(&self.device, "src/engine/shaders/shader.vert.spv");
    
    let vertex_shader_stage_create_info = VkPipelineShaderStageCreateInfo {
        s_type: 18,
        p_next: std::ptr::null(),
        flags: 0,
        stage: 0x00000001,
        module: vertex_shader,
        name: entry_point_name.as_ptr(),
        specialization_info: std::ptr::null()
    };

    let fragment_shader = create_shader_module_from_file(&self.device, "src/engine/shaders/shader.frag.spv");

    let fragment_shader_stage_create_info = VkPipelineShaderStageCreateInfo {
        s_type: 18,
        p_next: std::ptr::null(),
        flags: 0,
        stage: 0x00000010,
        module: fragment_shader,
        name: entry_point_name.as_ptr(),
        specialization_info: std::ptr::null()
    };

    let shader_stage_create_infos = [vertex_shader_stage_create_info, fragment_shader_stage_create_info];

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
        primitive_restart_enable: 0
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

    let color_blend_attachment = VkPipelineColorBlendAttachmentState {
        blend_enable: 0,
        src_color_blend_factor: 1,
        dst_color_blend_factor: 0,
        color_blend_op: 0,
        src_alpha_blend_factor: 1,
        dst_alpha_blend_factor: 0,
        alpha_blend_op: 0,
        color_write_mask: 0x00000001 | 0x00000002 | 0x00000004 | 0x00000008
    };

    let color_blend_attachments = [color_blend_attachment];

    let color_blend_state_create_info = VkPipelineColorBlendStateCreateInfo {
        s_type: 26,
        p_next: std::ptr::null(),
        flags: 0,
        logic_op_enable: 0,
        logic_op: 3,
        attachment_count: color_blend_attachments.len() as u32,
        attachments: color_blend_attachments.as_ptr(),
        blend_constants: [0.0, 0.0, 0.0, 0.0]
    };

    let pipeline_layout_create_info = VkPipelineLayoutCreateInfo {
        s_type: 30,
        p_next: std::ptr::null(),
        flags: 0,
        set_layout_count: 0,
        set_layouts: std::ptr::null(),
        push_constant_range_count: 0,
        push_constant_ranges: std::ptr::null()
    };

    vkCreatePipelineLayout(
        self.device.clone(),
        &pipeline_layout_create_info as *const VkPipelineLayoutCreateInfo,
        std::ptr::null(),
        &mut self.pipeline_layout
    );

    self.render_pass = create_render_pass(self.device.clone(), self.swapchain_image_format.format);

    let mut tessellation_state_create_info: VkPipelineTessellationStateCreateInfo = unsafe {std::mem::zeroed()};
    tessellation_state_create_info.s_type = 21;
    tessellation_state_create_info.p_next = std::ptr::null();
    
    let mut depth_stencil_create_info: VkPipelineDepthStencilStateCreateInfo = unsafe {std::mem::zeroed()};
    depth_stencil_create_info.s_type = 25;
    depth_stencil_create_info.p_next = std::ptr::null();

    let pipeline_create_info = VkGraphicsPipelineCreateInfo {
        s_type: 28,
        p_next: std::ptr::null(),
        flags: 0,
        stage_count: shader_stage_create_infos.len() as u32,
        stages: shader_stage_create_infos.as_ptr(),
        vertex_input_state: &vertex_input_state_create_info as *const VkPipelineVertexInputStateCreateInfo,
        input_assembly_state: &input_assembly_state_create_info as *const VkPipelineInputAssemblyStateCreateInfo,
        tesselation_state: &tessellation_state_create_info as *const VkPipelineTessellationStateCreateInfo,
        viewport_state: &viewport_state_create_info as *const VkPipelineViewportStateCreateInfo,
        rasterization_state: &rasterization_state_create_info as *const VkPipelineRasterizationStateCreateInfo,
        multisample_state: &multisample_state_create_info as *const VkPipelineMultisampleStateCreateInfo,
        depth_stencil_state: &depth_stencil_create_info as *const VkPipelineDepthStencilStateCreateInfo,
        color_blend_state: &color_blend_state_create_info as *const VkPipelineColorBlendStateCreateInfo,
        dynamic_state: std::ptr::null(),
        layout: self.pipeline_layout,
        render_pass: self.render_pass,
        subpass: 0,
        base_pipeline_handle: 0,
        base_pipeline_handle_index: -1
    };

    let pipeline_create_infos = [pipeline_create_info];

    vkCreateGraphicsPipelines(
        self.device.clone(),
        0,
        pipeline_create_infos.len() as u32,
        pipeline_create_infos.as_ptr(),
        std::ptr::null(),
        &mut self.pipeline
    );
}}}

fn create_render_pass(device: VkDevice, swapchain_image_format: u32) -> VkRenderPass {
    let attachment_description = VkAttachmentDescription {
        flags: 0,
        format: swapchain_image_format,
        samples: 0x00000001,
        load_op: 1,
        store_op: 0,
        stencil_load_op: 0,
        stencil_store_op: 0,
        initial_layout: 0,
        final_layout: 1000001002
    };
    
    let attachment_descriptions = [attachment_description];


    let color_attachment = VkAttachmentReference {
        attachment: 0,
        layout: 2
    };

    let color_attachments = [color_attachment];


    let subpass_description = VkSubpassDescription {
        flags: 0,
        pipeline_bind_point: 0,
        input_attachment_count: 0,
        input_attachments: std::ptr::null(),
        color_attachment_count: color_attachments.len() as u32,
        color_attachments: color_attachments.as_ptr(),
        resolve_attachments: std::ptr::null(),
        depth_stencil_attachment: std::ptr::null(),
        preserve_attachment_count: 0,
        preserve_attachments: std::ptr::null()
    };
    
    let subpass_descriptions = [subpass_description];


    let render_pass_create_info = VkRenderPassCreateInfo {
        s_type: 38,
        p_next: std::ptr::null(),
        flags: 0,
        attachment_count: attachment_descriptions.len() as u32,
        attachments: attachment_descriptions.as_ptr(),
        subpass_count: subpass_descriptions.len() as u32,
        subpasses: subpass_descriptions.as_ptr(),
        dependency_count: 0,
        dependencies: std::ptr::null()
    };
    

    let mut render_pass: VkRenderPass = 0;

    unsafe {vkCreateRenderPass(
        device,
        &render_pass_create_info as *const VkRenderPassCreateInfo,
        std::ptr::null(),
        &mut render_pass
    )};

    return render_pass;
}

fn create_shader_module_from_file(device: &VkDevice, file: &str) -> VkShaderModule {
    let mut file = std::fs::File::open(file).expect("Nonexisting shader file");

    let mut raw_code = vec!();
    file.read_to_end(&mut raw_code);

    let code: Vec<u32> = raw_code.chunks_exact(4).map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])).collect();

    let create_info = VkShaderModuleCreateInfo {
        s_type: 16,
        p_next: std::ptr::null(),
        flags: 0,
        code_size: code.len()*4,
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
