use crate::vulkan::{
    devices::{
        physical_device::{
            VkPhysicalDevice
        }
    },
    instance::{
        VkInstance
    }
};

use std::ffi::{
    c_void
};


pub type VkSurfaceKHR = u64;

#[derive(PartialEq)]
pub enum WindowEvent {
    Birth,
    Death,
    MetaChange,
    MouseDown(u32),
    MouseUp(u32),
    KeyUp(u32),
    KeyDown(u32),
    MouseMove(u32, u32),
    FocusChange(bool),
    WindowResize([i32; 2]),
    Undefined
}

pub trait Window {
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;

    fn set_width(&mut self, width: u32);
    fn set_height(&mut self, height: u32);

    fn init_connection(dimensions: [i32; 2]) -> Self where Self: Sized;
    
    fn init_window(&mut self, name: String);

    fn create_surfaceKHR(&self, instance: crate::vulkan::instance::VkInstance) -> VkSurfaceKHR;

    fn get_events(&self, dimensions: [i32; 2]) -> Vec<WindowEvent>;
    
    fn supports_physical_device_queue(&self, physical_device: VkPhysicalDevice, queue: u32) -> bool;

    fn commit_suicide(&self);
}

pub struct dummy {}

impl Window for dummy {
    fn get_width(&self) -> u32 {todo!();}
    fn get_height(&self) -> u32 {todo!();}

    fn set_width(&mut self, width: u32) {todo!();}
    fn set_height(&mut self, height: u32) {todo!();}

    fn init_connection(dimensions: [i32; 2]) -> Self where Self: Sized {todo!();}
    
    fn init_window(&mut self, name: String) {todo!();}

    fn create_surfaceKHR(&self, instance: crate::vulkan::instance::VkInstance) -> VkSurfaceKHR {todo!();}

    fn get_events(&self, dimensions: [i32; 2]) -> Vec<WindowEvent> {todo!();}
    
    fn supports_physical_device_queue(&self, physical_device: VkPhysicalDevice, queue: u32) -> bool {todo!();}

    fn commit_suicide(&self) {todo!();}
}

pub mod x11;

#[link(name = "vulkan")]
extern "C" {
    pub fn vkDestroySurfaceKHR(
        instance: VkInstance,
        surface: VkSurfaceKHR,
        _: *const c_void
    );
}
