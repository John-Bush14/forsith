mod initialisation;

mod loop_;

mod drop;

mod swapchain;

mod image;

pub mod interface;

mod test;

pub mod world_view;


use crate::engine::interface::drawables::Drawable;


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
        physical_device::VkPhysicalDevice
    },
    window::{
        VkSurfaceKHR,
        Window,
        WindowEvent
    },
    swapchain::{
        image_view::VkImageView,
        VkSwapchainKHR,
        VkSurfaceFormatKHR,
        VkImage,
        VkExtent2D
    },
    pipeline::{
        GraphicsPipeline,
        VkPipelineLayout,
        VkRenderPass,
        VkFramebuffer
    },
    commands::{
        command_buffer::VkCommandBuffer,
        command_pool::VkCommandPool
    },
    rendering::{
        VkSemaphore,
        VkFence
    },
    vertex::{
        Vertex,
        VkBuffer,
        VkDeviceMemory
    },
    uniform::{
        VkDescriptorPool,
        VkDescriptorSetLayout
    }
};


pub enum Event {}

pub struct Engine {
    app_name: String,
    app_version: u32,
    instance: VkInstance,
    device: VkDevice,
    physical_device: VkPhysicalDevice,
    surface_khr: VkSurfaceKHR,
    pub window: Box<dyn Window>,
    swapchain: VkSwapchainKHR,
    swapchain_image_format: VkSurfaceFormatKHR,
    swapchain_images: Vec<VkImage>,
    swapchain_extent: VkExtent2D,
    swapchain_image_views: Vec<VkImageView>,
    pipeline_layouts: std::collections::HashMap<u32, (VkPipelineLayout, VkDescriptorSetLayout)>,
    render_pass: VkRenderPass,
    pipelines: Vec<GraphicsPipeline>,
    framebuffers: Vec<VkFramebuffer>,
    _debug_report_callback: VkDebugUtilsMessengerEXT,
    command_pool: VkCommandPool,
    transient_command_pool: VkCommandPool,
    command_buffers: Vec<VkCommandBuffer>,
    image_available_semaphores: Vec<VkSemaphore>,
    render_finished_semaphores: Vec<VkSemaphore>,
    in_flight_fences: Vec<VkFence>,
    current_frame: usize,
    graphics_queue: VkQueue,
    presentation_queue: VkQueue,
    graphics_family: u32,
    presentation_family: u32,
    pub dimensions: [i32; 2],
    new_dimensions: Option<[i32; 2]>,
    vertex_buffer: VkBuffer,
    vertex_buffer_memory: VkDeviceMemory,
    descriptor_pool: VkDescriptorPool,
    vertices: Vec<Vertex>,
    vertex_usage_counts: std::collections::HashMap<Vertex, usize>, // <Vertex, usage count>
    vertex_indices: std::collections::HashMap<Vertex, u16>, // <Vertex, Indice>
    drawables: Vec<Drawable>,
    world_view: world_view::WorldView,
    pub events: Vec<WindowEvent>,
    pub target_fps: f32,
}

pub fn initialize_engine<T>(
    name: String,
    version: [u8;3], 
    mut user_data: T,
    ready_func: fn(&mut Engine, &mut T), 
    event_loop: fn(&mut Engine, &mut T, f32)
) {
    let mut engine = Engine::init(name, version).expect("Initialisation of engine failed");
    
    ready_func(&mut engine, &mut user_data);

    engine.start_loop(event_loop, user_data);
}
