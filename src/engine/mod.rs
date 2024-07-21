mod initialisation;
use initialisation::{};

mod loop_;

mod drop;

mod swapchain;


use crate::vulkan::{
    instance::{
        VkInstance, 
        VkDebugUtilsMessengerEXT
    },
    devices::{
        device::{
            VkDevice,
            VkQueue
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
    },
    commands::{
        command_buffer::{
            VkCommandBuffer
        },
        command_pool::{
            VkCommandPool
        }
    },
    rendering::{
        VkSemaphore,
        VkFence
    }
};

use core::ffi::c_void;

pub enum Event {}

pub struct Engine {
    app_name: String,
    app_version: u32,
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
    debug_report_callback: VkDebugUtilsMessengerEXT,
    command_pool: VkCommandPool,
    command_buffers: Vec<VkCommandBuffer>,
    image_available_semaphores: Vec<VkSemaphore>,
    render_finished_semaphores: Vec<VkSemaphore>,
    in_flight_fences: Vec<VkFence>,
    current_frame: usize,
    graphics_queue: VkQueue,
    presentation_queue: VkQueue,
    graphics_family: u32,
    presentation_family: u32,
    dimensions: [i32; 2],
    new_dimensions: Option<[i32; 2]>
}

static mut ENGINE: Option<Engine> = None;

pub fn initialize_engine(name: String, version: [u8;3], event_loop: fn()) {
    let mut engine = unsafe {Engine::init(name, version, event_loop).expect("Initialisation of engine failed")};
    unsafe {engine.start_loop()};
}
