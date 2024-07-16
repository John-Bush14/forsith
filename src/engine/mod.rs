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
        },
        physical_device::{
            VkPhysicalDevice
        }
    },
    window::{
        VkSurfaceKHR,
        Window,
        WindowEvent
    },
    swapchain::{
        VkSwapchainKHR,
        VkImage,
        VkExtent2D
    }
};

use core::ffi::c_void;

pub enum Event {}

pub struct Engine {
    app_name: String,
    app_version: u32,
    event_func: fn(),
    instance: VkInstance,
    device: VkDevice,
    physical_device: VkPhysicalDevice,
    surface_khr: VkSurfaceKHR,
    window: Box<dyn Window>,
    swapchain: VkSwapchainKHR,
    swapchain_image_format: u32,
    swapchain_images: Vec<VkImage>,
    swapchain_extent: VkExtent2D
}

static mut ENGINE: Option<Engine> = None;

pub fn initialize_engine(name: String, version: [u8;3], event_loop: fn()) {
    unsafe { ENGINE = Some(Engine::init(name, version, event_loop).expect("Initialisation of engine failed")); }
}
