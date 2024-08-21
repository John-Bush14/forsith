use crate::vulkan::{
    devices::{device::VkDevice, physical_device::{vkGetPhysicalDeviceFormatProperties, VkFormatProperties}}, pipeline::{
        vkCreateFramebuffer, vkCreateGraphicsPipelines, vkCreatePipelineLayout, vkCreateRenderPass, vkCreateShaderModule, GraphicsPipeline, Uniform, VkAttachmentDescription, VkAttachmentReference, VkFramebuffer, VkFramebufferCreateInfo, VkGraphicsPipelineCreateInfo, VkOffset2D, VkPipeline, VkPipelineColorBlendAttachmentState, VkPipelineColorBlendStateCreateInfo, VkPipelineDepthStencilStateCreateInfo, VkPipelineInputAssemblyStateCreateInfo, VkPipelineLayoutCreateInfo, VkPipelineMultisampleStateCreateInfo, VkPipelineRasterizationStateCreateInfo, VkPipelineShaderStageCreateInfo, VkPipelineTessellationStateCreateInfo, VkPipelineVertexInputStateCreateInfo, VkPipelineViewportStateCreateInfo, VkRect2D, VkRenderPass, VkRenderPassCreateInfo, VkShaderModule, VkShaderModuleCreateInfo, VkStencilOpState, VkSubpassDependency, VkSubpassDescription, VkViewport
    }, uniform::DescriptorBindings, vertex::{
        Vertex, VkVertexInputAttributeDescription, VkVertexInputBindingDescription
    }
};

use std::io::Read;

use std::ffi::CString;


#[allow(dead_code)]
pub const PIPELINE_3D: usize = 0;

#[allow(dead_code)]
pub const PIPELINE_2D: usize = 1;

#[allow(dead_code)]
pub const PIPELINE_UI_3D: usize = 2;

#[allow(dead_code)]
pub const PIPELINE_UI_2D: usize = 3;


impl Uniform {
    pub fn size_of(&self) -> u64 {
        use std::mem::size_of as szf;

        return match self {
            Self::Model3d => {szf::<[[f32;4];4]>()}
            Self::Model2d => {szf::<[[f32;4];4]>()},
            Uniform::Camera2d => {szf::<([[f32;4];4], f32)>()},
            Uniform::Camera3d => {szf::<([[f32;4];4], [[f32;4];4])>()},
            Uniform::Image => {0} // special
        } as u64;
    }
}
                                                            
impl GraphicsPipeline { pub fn new(
    vertex_shader: &str,
    fragment_shader: &str, 
    vertex_input: Vec<Uniform>, 
    fragment_input: Vec<Uniform>,
    device: &VkDevice, 
    persistent: bool
) -> GraphicsPipeline {
    
    return GraphicsPipeline {
        pipeline: 0,
        vertex_shader: create_shader_module_from_file(device, vertex_shader),
        fragment_shader: create_shader_module_from_file(device, fragment_shader),
        vertex_uniforms: vertex_input,
        fragment_uniforms: fragment_input,
        uses: {if persistent {1} else {0}}
    }
}}

impl crate::engine::Engine { pub fn default_pipelines(&self) -> Vec<GraphicsPipeline> {
    return vec![
        GraphicsPipeline::new(
            "src/engine/shaders/3d/shader.vert.spv",
	         "src/engine/shaders/shader.frag.spv",
            vec![Uniform::Camera3d, Uniform::Model3d],
            vec!(),
            &self.device,
            true
        ),
        GraphicsPipeline::new(
            "src/engine/shaders/2d/shader.vert.spv",
	         "src/engine/shaders/shader.frag.spv",
            vec![Uniform::Camera2d, Uniform::Model2d],
            vec!(),
            &self.device,
            true
        ),
        GraphicsPipeline::new(
            "src/engine/shaders/ui/3d/shader.vert.spv",
	         "src/engine/shaders/shader.frag.spv",
            vec![Uniform::Camera3d, Uniform::Model2d],
            vec!(),
            &self.device,
            false
        ),
        GraphicsPipeline::new(
            "src/engine/shaders/ui/2d/shader.vert.spv",
	         "src/engine/shaders/shader.frag.spv",
            vec![Uniform::Camera2d, Uniform::Model2d],
            vec!(),
            &self.device,
            true
        ),
        GraphicsPipeline::new(
            "src/engine/shaders/image.vert.spv",
	         "src/engine/shaders/image.frag.spv",
            vec![Uniform::Camera2d, Uniform::Model2d],
            vec![Uniform::Image],
            &self.device,
            true
        )
    ];
}}

impl DescriptorBindings {pub fn from(vertex_uniforms: Vec<Uniform>, fragment_uniforms: Vec<Uniform>) -> Self {                 
    return DescriptorBindings(
        vertex_uniforms.iter().map(|uniform| return match *uniform {Uniform::Image => 1, _ => 0}).collect(),
        fragment_uniforms.iter().map(|uniform| return match *uniform {Uniform::Image => 1, _ => 0}).collect()
    )
}}

impl crate::engine::Engine {pub fn create_pipeline_layouts(&mut self) {
    self.pipelines.iter()                                                                                                                                                                           
        .map(|pipeline| return DescriptorBindings::from(pipeline.vertex_uniforms.clone(), pipeline.fragment_uniforms.clone()))
        .filter(|descriptor_binding| return !self.pipeline_layouts.contains_key(&descriptor_binding))
        .collect::<std::collections::HashSet<_>>().into_iter()
        .collect::<Vec<_>>().into_iter()
        .for_each(|descriptor_count| {
            let descriptor_set_layout = self.create_descriptor_set_layout(descriptor_count.clone());

            let pipeline_layout_create_info = VkPipelineLayoutCreateInfo {
	             s_type: 30,
		          p_next: std::ptr::null(),
	 	          flags: 0,
	 	          set_layout_count: 1,
		          set_layouts: &descriptor_set_layout as *const u64,
		          push_constant_range_count: 0,
	 	          push_constant_ranges: std::ptr::null()
	         };

            let mut pipeline_layout = 0;

            unsafe { vkCreatePipelineLayout(
		          self.device.clone(),
		          &pipeline_layout_create_info as *const VkPipelineLayoutCreateInfo,
	 	          std::ptr::null(),
		          &mut pipeline_layout
            )};

            self.pipeline_layouts.insert(descriptor_count, (pipeline_layout, descriptor_set_layout));
    });
}}

impl crate::engine::Engine {pub fn create_framebuffers(&mut self) {
    self.framebuffers = self.swapchain_image_views.iter().map(|view| [*view, self.depth_image.2]).map(|attachments| {
        let framebuffer_create_info = VkFramebufferCreateInfo {
            s_type: 37,
            p_next: std::ptr::null(),
            flags: 0,
            render_pass: self.render_pass,
            attachment_count: attachments.len() as u32,
            attachments: attachments.as_ptr(),
            width: self.swapchain_extent.width,
            height: self.swapchain_extent.height,
            layers: 1
        };

        let mut framebuffer: VkFramebuffer = 0;

        unsafe {vkCreateFramebuffer(
            self.device,
            &framebuffer_create_info as *const VkFramebufferCreateInfo,
            std::ptr::null(),
            &mut framebuffer
        )};

        return framebuffer;
    }).collect();
}}
                    
impl crate::engine::Engine {pub fn create_pipeline(&self, pipeline: &GraphicsPipeline) -> VkPipeline {
    let entry_point_name = CString::new("main").unwrap();
    let descriptor_bindings = DescriptorBindings::from(pipeline.vertex_uniforms.clone(), pipeline.fragment_uniforms.clone());
    let pipeline_layout = self.pipeline_layouts.get(&descriptor_bindings).unwrap().0;

    let vertex_shader_stage_create_info = VkPipelineShaderStageCreateInfo {
		  s_type: 18,
		  p_next: std::ptr::null(),
		  flags: 0,
		  stage: 0x00000001,
		  module: pipeline.vertex_shader,
		  name: entry_point_name.as_ptr(),
		  specialization_info: std::ptr::null()
	 };

	 let fragment_shader_stage_create_info = VkPipelineShaderStageCreateInfo {
		  s_type: 18,
		  p_next: std::ptr::null(),
		  flags: 0,
		  stage: 0x00000010,
		  module: pipeline.fragment_shader,
		  name: entry_point_name.as_ptr(),
		  specialization_info: std::ptr::null()
	 };
    
	 let shader_stage_create_infos = [vertex_shader_stage_create_info, fragment_shader_stage_create_info];

	 let vertex_input_binding_description = VkVertexInputBindingDescription {
		  binding: 0,
		  stride: std::mem::size_of::<Vertex>() as u32,
		  input_rate: 0,
	 };

	 let position_vertex_input_attribute_description = VkVertexInputAttributeDescription {
		  location: 0,
		  binding: 0,
		  format: 106,
		  offset: 0
	 };

	 let color_vertex_input_attribute_description = VkVertexInputAttributeDescription {
		  location: 1,
		  binding: 0,
		  format: 109,
		  offset: 12
	 };
	 
    let coords_vertex_input_attribute_description = VkVertexInputAttributeDescription {
		  location: 2,
		  binding: 0,
		  format: 103,
		  offset: 28
	 };

	 let vertex_binding_descriptions = [vertex_input_binding_description];

    let mut vertex_attribute_descriptions = vec![position_vertex_input_attribute_description, color_vertex_input_attribute_description];
                
    if descriptor_bindings.0.contains(&1) || descriptor_bindings.1.contains(&1) {vertex_attribute_descriptions.push(coords_vertex_input_attribute_description);}

	 let vertex_input_state_create_info = VkPipelineVertexInputStateCreateInfo {
		  s_type: 19,
		  p_next: std::ptr::null(),
		  flags: 0,
		  vertex_binding_description_count: vertex_binding_descriptions.len() as u32,
		  vertex_binding_descriptions: vertex_binding_descriptions.as_ptr(),
		  vertex_attribute_description_count: vertex_attribute_descriptions.len() as u32,
		  vertex_attribute_descriptions: vertex_attribute_descriptions.as_ptr()
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
		  front_face: 0,
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
		  alpha_to_coverage_enable: 1,
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

	 let mut tessellation_state_create_info: VkPipelineTessellationStateCreateInfo = unsafe {std::mem::zeroed()};
	 tessellation_state_create_info.s_type = 21;
	 tessellation_state_create_info.p_next = std::ptr::null();

    let default_stencil_op_state: VkStencilOpState = unsafe {std::mem::zeroed()};
	 	
	 let depth_stencil_create_info = VkPipelineDepthStencilStateCreateInfo {
        s_type: 25,
        p_next: std::ptr::null(),
        flags: 0,
        depth_test_enable: 1, 
        depth_write_enable: 1,
        depth_compare_op: 1,
        depth_bounds_test_enable: 0,
        stencil_test_enable: 0,
        front: default_stencil_op_state.clone(), back: default_stencil_op_state,
        min_depth_bounds: 0.0,
        max_depth_bounds: 1.0 
    };

	 let create_info = VkGraphicsPipelineCreateInfo {
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
		  layout: pipeline_layout,
		  render_pass: self.render_pass,
		  subpass: 0,
        base_pipeline_handle: 0,
		  base_pipeline_handle_index: -1
	 };

    let create_infos = [create_info];
    
    let mut pipeline = 0;

    unsafe { vkCreateGraphicsPipelines(
		  self.device.clone(),
		  0,
	     create_infos.len() as u32,                                                                                                                                                                                                                                     
		  create_infos.as_ptr(),
		  std::ptr::null(),
		  &mut pipeline
	 )};

    return pipeline;
}}

impl crate::engine::Engine {pub fn find_depth_format(&mut self) {
    let tiling = 0;
    let features = 0x00000200;

    let candidates = [
        126,
        130,
        129
    ];

    self.depth_format = candidates.into_iter().find(|candidate| {
        let mut properties: VkFormatProperties = unsafe {std::mem::zeroed()};
                                
        unsafe {vkGetPhysicalDeviceFormatProperties(self.physical_device, *candidate, &mut properties as *mut VkFormatProperties)};

        return (tiling == 1 && (properties.linear_tiling_features & features != 0)) || (tiling == 0)
    }).expect("no supported depth format found!");
}}

pub fn create_render_pass(device: VkDevice, swapchain_image_format: u32, depth_format: u32) -> VkRenderPass {
    let color_attachment_description = VkAttachmentDescription {
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

    let depth_attachment_description = VkAttachmentDescription {
        flags: 0,
        format: depth_format,
        samples: 0x00000001,
        load_op: 1,
        store_op: 1,
        stencil_load_op: 2,
        stencil_store_op: 1,
        initial_layout: 0,
        final_layout: 3
    };
    
    let attachment_descriptions = [color_attachment_description, depth_attachment_description];


    let color_attachment = VkAttachmentReference {
        attachment: 0,
        layout: 2
    };
    
    let depth_attachment = VkAttachmentReference {
        attachment: 1,
        layout: 3
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
        depth_stencil_attachment: &depth_attachment as *const VkAttachmentReference,
        preserve_attachment_count: 0,
        preserve_attachments: std::ptr::null()
    };
    
    let subpass_descriptions = [subpass_description];
    

    let dependency = VkSubpassDependency {
        src_subpass: std::u32::MAX,
        dst_subpass: 0,
        src_stage_mask: 0x00000400,
        dst_stage_mask: 0x00000400,
        src_acces_mask: 0,
        dst_acces_mask: 0x00000080 | 0x00000100,
        dependency_flags: 0
    };

    let dependencies = [dependency];


    let render_pass_create_info = VkRenderPassCreateInfo {
        s_type: 38,
        p_next: std::ptr::null(),
        flags: 0,
        attachment_count: attachment_descriptions.len() as u32,
        attachments: attachment_descriptions.as_ptr(),
        subpass_count: subpass_descriptions.len() as u32,
        subpasses: subpass_descriptions.as_ptr(),
        dependency_count: dependencies.len() as u32,
        dependencies: dependencies.as_ptr()
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
    let _ = file.read_to_end(&mut raw_code);

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
