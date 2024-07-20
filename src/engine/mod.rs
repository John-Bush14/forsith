mod initialisation;
use initialisation::{};

mod drop;

use crate::vulkan::{
    instance::{
        VkInstance, 
        VkDebugUtilsMessengerEXT
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
        image_view::{
            VkImageView
        },
        VkSwapchainKHR,
        VkSurfaceFormatKHR,
        VkImage,
        VkExtent2D
    },
    pipeline::{
        VkPipelineLayout,
        VkRenderPass,
        VkPipeline,
        VkShaderModule,
        VkFramebuffer
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
    swapchain_image_format: VkSurfaceFormatKHR,
    swapchain_images: Vec<VkImage>,
    swapchain_extent: VkExtent2D,
    swapchain_image_views: Vec<VkImageView>,
    pipeline_layout: VkPipelineLayout,
    render_pass: VkRenderPass,
    pipeline: VkPipeline,
    shader_modules: Vec<VkShaderModule>,
    framebuffers: Vec<VkFramebuffer>,
    debug_report_callback: VkDebugUtilsMessengerEXT
}

static mut ENGINE: Option<Engine> = None;

pub fn initialize_engine(name: String, version: [u8;3], event_loop: fn()) {
    unsafe { ENGINE = Some(Engine::init(name, version, event_loop).expect("Initialisation of engine failed")); }
}
