mod instance;
mod device;
mod window;
mod swapchain;
mod pipeline;
mod command_buffers;
mod sync_objects;
mod uniform;
pub mod buffer;


use crate::vulkan::{
    instance::{
        vkEnumerateInstanceExtensionProperties, VkExtensionProperties
    }, pipeline::Uniform, vk_make_version, window::Dummy
};

use crate::vk_enumerate_to_vec;

use super::world_view::WorldView;


impl super::Engine {
    pub fn init(name: String, version: [u8;3]) -> Result<Self, Box<dyn std::error::Error>> { 
        let mut engine: super::Engine = super::Engine {
            app_name: name.clone(),
            app_version: vk_make_version(version[0] as u32, version[1] as u32, version[2] as u32),
            vertices: vec!(),
            instance: 0,
            device: 0,
            physical_device: 0,
            surface_khr: 0,
            window: Box::new(Dummy {}),
            swapchain: unsafe {std::mem::zeroed()},
            swapchain_image_format: unsafe {std::mem::zeroed()},
            swapchain_images: vec!(),
            swapchain_extent: unsafe {std::mem::zeroed()},
            swapchain_image_views: vec!(),
            pipeline_layouts: std::collections::HashMap::new(),
            render_pass: 0,
            pipelines: vec!(),
            _debug_report_callback: 0,
            framebuffers: vec!(),
            command_pool: 0,
            transient_command_pool: 0,
            command_buffers: vec!(),
            image_available_semaphores: vec!(), 
            render_finished_semaphores: vec!(),
            in_flight_fences: vec!(),
            current_frame: 0,
            graphics_queue: 0,
            presentation_queue: 0,
            graphics_family: 0,
            presentation_family: 0,
            dimensions: [800, 600],
            new_dimensions: None,
            vertex_buffer: 0,
            vertex_buffer_memory: 0,
            descriptor_pool: 0,
            vertex_usage_counts: std::collections::HashMap::new(),
            vertex_indices: std::collections::HashMap::new(),
            drawables: vec!(),                  
            world_view: WorldView::zero(),
            events: vec!(),
            target_fps: 0.0
        };

        let supported_instance_extensions = unsafe { vk_enumerate_to_vec!(
            vkEnumerateInstanceExtensionProperties, 
            VkExtensionProperties,
            std::ptr::null(),
        )};


        engine.create_instance(supported_instance_extensions.clone());


        let test_window_connections = engine.create_test_connections(supported_instance_extensions);

        let chosen_window_connection = engine.create_device(test_window_connections);
    

        engine.finalize_connection(chosen_window_connection, engine.app_name.clone());
        
        engine.create_surface_khr(engine.instance);


        engine.create_swapchain();

        engine.create_image_views();
        
        
        let (uniform_buffers, uniform_memories) = engine.create_uniform_buffers(Uniform::Camera2d.size_of());
        let uniform_buffers_2d = uniform_buffers.iter()
            .zip(uniform_memories.iter())
            .map(|(x, y)| (*x, *y))
            .collect::<Vec<(_, _)>>();

        let (uniform_buffers, uniform_memories) = engine.create_uniform_buffers(Uniform::Camera3d.size_of());
        let uniform_buffers_3d = uniform_buffers.iter()
            .zip(uniform_memories.iter())
            .map(|(x, y)| (*x, *y))
            .collect::<Vec<(_, _)>>();

        let uniform_buffers = vec![uniform_buffers_3d, uniform_buffers_2d];

        engine.world_view = super::world_view::WorldView::new(
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 0.0],
            60.0,
            0.1,
            10.0,
            uniform_buffers
        );


        engine.add_pipelines(engine.default_pipelines());

        engine.create_pipelines(false);


        engine.create_command_pool(false);
        
        engine.create_command_pool(true);
        
        engine.create_descriptor_pool();

        
        return Ok(engine);
    }
}
