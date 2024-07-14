mod initialisation;
use initialisation::{};

mod drop;

use crate::vulkan::{
    instance::{
        VkInstance
    },
    devices::{
        device::{
            VkDevice
        }
    },
    window::{
        VkSurfaceKHR,
        Window,
        WindowEvent
    }
};

use core::ffi::c_void;

pub enum Event {}

pub struct Engine {
    app_name: String,
    app_version: u32,
    event_func: fn(WindowEvent),
    instance: VkInstance,
    device: VkDevice,
    surface_khr: VkSurfaceKHR,
    window: Box<dyn Window>
}

static mut ENGINE: Option<Engine> = None;

pub fn initialize_engine(name: String, version: [u8;3], event_loop: fn(WindowEvent)) {
    unsafe { ENGINE = Some(Engine::init(name, version, event_loop).expect("Initialisation of engine failed")); }
}
