mod instance;
mod device;
mod window;

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
            event_func: event_loop,
            instance: 0,
            device: 0,
            surface_khr: 0,
            window: Box::new(dummy {})
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

        engine.window.start_loop(event_loop);


        return Ok(engine);
    };}
}
