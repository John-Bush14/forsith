mod instance;
mod device;
mod window;
mod swapchain;
mod pipeline;
mod command_buffers;
mod sync_objects;


use crate::vulkan::{
    instance::{
        VkExtensionProperties,
        vkEnumerateInstanceExtensionProperties
    },
    window::{
        VkSurfaceKHR,
        Window,
        dummy,
        WindowEvent
    },
    vk_make_version,
};

use crate::{
    vk_enumerate_to_vec
};


impl super::Engine {
    pub fn init(name: String, version: [u8;3], event_loop: fn()) -> Result<Self, Box<dyn std::error::Error>> { unsafe {
        let mut engine: super::Engine = super::Engine {
            app_name: name.clone(),
            app_version: vk_make_version(version[0] as u32, version[1] as u32, version[2] as u32),
            instance: 0,
            device: 0,
            physical_device: 0,
            surface_khr: 0,
            window: Box::new(dummy {}),
            swapchain: std::mem::zeroed(),
            swapchain_image_format: std::mem::zeroed(),
            swapchain_images: vec!(),
            swapchain_extent: std::mem::zeroed(),
            swapchain_image_views: vec!(),
            pipeline_layout: 0,
            render_pass: 0,
            shader_modules: vec!(),
            pipeline: 0,
            debug_report_callback: 0,
            framebuffers: vec!(),
            command_pool: 0,
            command_buffers: vec!(),
            image_available_semaphores: vec!(),
            render_finished_semaphores: vec!(),
            in_flight_fences: vec!(),
            current_frame: 0,
            graphics_queue: 0,
            presentation_queue: 0,
            graphics_family: 0,
            presentation_family: 0
        };


        let supported_instance_extensions = vk_enumerate_to_vec!(
            vkEnumerateInstanceExtensionProperties, 
            VkExtensionProperties,
            std::ptr::null(),
        );


        engine.create_instance(supported_instance_extensions.clone());


        let mut test_window_connections = super::Engine::create_test_connections(supported_instance_extensions);

        let chosen_window_connection = engine.create_device(test_window_connections);


        engine.finalize_connection(chosen_window_connection, engine.app_name.clone());
        
        engine.create_surface_KHR(engine.instance);


        engine.create_swapchain();

        engine.create_image_views();

        engine.create_pipeline();


        engine.create_command_pool();
        
        engine.create_command_buffers();


        engine.create_sync_objects();


        return Ok(engine);
    };}
}
