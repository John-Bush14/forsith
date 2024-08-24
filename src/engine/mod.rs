pub(crate) mod initialisation;

mod loop_;

mod drop;

mod swapchain;

mod commands;

mod image;

pub mod drawables;

pub mod drawable_interface;

mod test;

pub mod world_view;

mod pipelines;


use crate::engine::drawables::Drawable;


use crate::vulkan::image::Texture;
use crate::vulkan::uniform::DescriptorBindings;
use crate::vulkan::vertex::{vkMapMemory, vkUnmapMemory};
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
        VkSwapchainKHR,
        VkSurfaceFormatKHR,
        VkExtent2D
    },
    image::{
        VkImage,
        VkImageView
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


pub struct Engine {
    app_name: String,
    app_version: u32,
    instance: VkInstance,
    device: VkDevice,
    physical_device: VkPhysicalDevice,
    surface_khr: VkSurfaceKHR,
    pub(crate) window: Box<dyn Window>,
    swapchain: VkSwapchainKHR,
    swapchain_image_format: VkSurfaceFormatKHR,
    swapchain_images: Vec<VkImage>,
    swapchain_extent: VkExtent2D,
    swapchain_image_views: Vec<VkImageView>,
    pipeline_layouts: std::collections::HashMap<DescriptorBindings, (VkPipelineLayout, VkDescriptorSetLayout)>,
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
    pub(crate) dimensions: [i32; 2],
    new_dimensions: Option<[i32; 2]>,
    vertex_buffer: VkBuffer,
    vertex_buffer_memory: VkDeviceMemory,
    descriptor_pool: VkDescriptorPool,
    vertices: Vec<Vertex>,
    vertex_usage_counts: std::collections::HashMap<Vertex, usize>, // <Vertex, usage count>
    vertex_indices: std::collections::HashMap<Vertex, u16>, // <Vertex, Indice>
    drawables: Vec<Drawable>,
    pub(crate) world_view: world_view::WorldView,
    pub(crate) events: Vec<WindowEvent>,
    pub target_fps: f32,
    depth_texture: Texture,
    depth_format: u32,
    msaa_samples: u32,
    color_texture: Texture
}

impl Engine {
    pub fn get_events(&self) -> &Vec<WindowEvent> {return &self.events}

    pub fn get_world_view(&self) -> &world_view::WorldView {return &self.world_view}

    pub fn get_dimensions(&self) -> &[i32;2] {return &self.dimensions}

    pub fn get_window(&self) -> &Box<dyn Window> {return &self.window}
}

pub fn initialize_engine<T>(
    name: String,
    version: [u8;3],
    ready_func: fn(&mut Engine) -> T,
    event_loop: fn(&mut Engine, &mut T, f32)
) {
    let mut engine = Engine::init(name, version).expect("Initialisation of engine failed");

    let user_data = ready_func(&mut engine);

    engine.start_loop(event_loop, user_data);
}

pub fn update_memory<T>(
    buffer_memory: VkDeviceMemory,
    device: u64,
    data: T,
) {
    let size = std::mem::size_of::<T>() as u64;

    let mut data_ptr: *mut std::ffi::c_void = std::ptr::null_mut();

    unsafe {vkMapMemory(
        device,
        buffer_memory,
        0,
        size,
        0,
        &mut data_ptr as _
    )};

    let data_arr = [data];

    unsafe {std::ptr::copy_nonoverlapping(data_arr.as_ptr(), data_ptr as _, data_arr.len())};

    unsafe {vkUnmapMemory(device, buffer_memory)};
}

