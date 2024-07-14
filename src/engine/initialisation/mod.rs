mod instance;
mod device;
mod window;

use crate::vulkan::{
    vk_make_version,
    window::{
        VkSurfaceKHR,
        Window,
        WindowEvent
    }
};

impl super::Engine {
    pub fn init(name: String, version: [u8;3], event_loop: fn(WindowEvent)) -> Result<Self, Box<dyn std::error::Error>> { unsafe {
        let mut engine: super::Engine = super::Engine {
            app_name: name.clone(),
            app_version: vk_make_version(version[0] as u32, version[1] as u32, version[2] as u32),
            event_func: event_loop,
            instance: 0,
            device: 0,
            surface_khr: 0,
            window: super::Engine::create_window(name.clone())
        };

        engine.create_instance();
        
        engine.create_surface_KHR(engine.instance);

        engine.create_device();

        engine.window.start_loop(event_loop);

        return Ok(engine);
    };}
}
