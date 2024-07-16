use crate::vulkan::{
    devices::{
        physical_device::{
            VkPhysicalDevice
        }
    }
};


pub type VkSurfaceKHR = u64;

pub enum WindowEvent {
    Birth,
    Death,
    MetaChange,
    MouseDown,
    MouseUp,
    KeyUp,
    KeyDown,
    MouseMove,
    FocusChange,
}

pub trait Window {
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;

    fn set_width(&mut self, width: u32);
    fn set_height(&mut self, height: u32);

    fn init_connection() -> Self where Self: Sized;
    
    fn init_window(&self, name: String);

    fn create_surfaceKHR(&self, instance: crate::vulkan::instance::VkInstance) -> VkSurfaceKHR;

    fn start_loop(&self, function: fn());
    
    fn supports_physical_device_queue(&self, physical_device: VkPhysicalDevice, queue: u32) -> bool;
}

pub struct dummy {}

impl Window for dummy {
    fn get_width(&self) -> u32 {todo!();}
    fn get_height(&self) -> u32 {todo!();}

    fn set_width(&mut self, width: u32) {todo!();}
    fn set_height(&mut self, height: u32) {todo!();}

    fn init_connection() -> Self where Self: Sized {todo!();}
    
    fn init_window(&self, name: String) {todo!();}

    fn create_surfaceKHR(&self, instance: crate::vulkan::instance::VkInstance) -> VkSurfaceKHR {todo!();}

    fn start_loop(&self, function: fn()) {todo!();}
    
    fn supports_physical_device_queue(&self, physical_device: VkPhysicalDevice, queue: u32) -> bool {todo!();}
}

pub mod x11;
