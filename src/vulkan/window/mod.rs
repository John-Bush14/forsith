use crate::vulkan::{
    devices::physical_device::VkPhysicalDevice,
    instance::VkInstance
};

use std::ffi::c_void;


pub type VkSurfaceKHR = u64;

#[derive(PartialEq, Clone)]
pub enum WindowEvent {
    Birth,
    Death,
    MetaChange,
    MouseDown(u32),
    MouseUp(u32),
    KeyUp(u32),
    KeyDown(u32, bool),
    MouseMove(f32, f32),
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

    fn init_window(&mut self, name: &str);

    fn create_surface_khr(&self, instance: crate::vulkan::instance::VkInstance) -> VkSurfaceKHR;

    fn get_events(&mut self, dimensions: [i32; 2]) -> Vec<WindowEvent>;

    fn supports_physical_device_queue(&self, physical_device: VkPhysicalDevice, queue: u32) -> bool;

    fn commit_suicide(&self);

    fn set_mouse(&mut self, x: f32, y: f32);
}

pub struct Dummy {}

impl Window for Dummy {
    fn get_width(&self) -> u32 {todo!();}
    fn get_height(&self) -> u32 {todo!();}

    fn set_width(&mut self, _width: u32) {todo!();}
    fn set_height(&mut self, _height: u32) {todo!();}

    fn init_connection(_dimensions: [i32; 2]) -> Self where Self: Sized {todo!();}

    fn init_window(&mut self, _name: &str) {todo!();}

    fn create_surface_khr(&self, _instance: crate::vulkan::instance::VkInstance) -> VkSurfaceKHR {todo!();}

    fn get_events(&mut self, _dimensions: [i32; 2]) -> Vec<WindowEvent> {todo!();}

    fn supports_physical_device_queue(&self, _physical_device: VkPhysicalDevice,_queuee: u32) -> bool {todo!();}

    fn commit_suicide(&self) {todo!();}

    fn set_mouse(&mut self, _x: f32, _y: f32) {todo!();}
}

#[cfg(target_os = "linux")]
pub mod x11;

#[cfg(target_os = "linux")]
pub mod wayland;

#[cfg(target_os = "windows")]
pub mod win32;

#[cfg(target_os = "macos")]
pub mod metal;


#[link(name = "vulkan")]
extern "C" {
    pub fn vkDestroySurfaceKHR(
        instance: VkInstance,
        surface: VkSurfaceKHR,
        _: *const c_void
    );
}
