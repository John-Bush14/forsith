pub type VkSurfaceKHR = u64;

pub enum WindowEvent {test}

pub trait Window {
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;

    fn set_width(&mut self, width: u32);
    fn set_height(&mut self, height: u32);

    fn init(name: String) -> Self where Self: Sized;

    fn create_surfaceKHR(&self, instance: crate::vulkan::instance::VkInstance) -> VkSurfaceKHR;

    fn start_loop(&self, function: fn(event: WindowEvent));
}

pub mod x11;
