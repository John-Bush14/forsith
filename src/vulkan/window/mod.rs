use crate::vulkan::{
    devices::physical_device::VkPhysicalDevice,
    instance::VkInstance
};

use std::ffi::c_void;


pub type VkSurfaceKHR = u64;

/// possible window events
#[derive(PartialEq, Clone)]
pub enum WindowEvent {
    /// window creation
    Birth,

    /// window death
    Death,

    /// change in some window property
    MetaChange,

    /// mouse button pressed, `u32` = button keycode
    MouseDown(u32),

    /// mouse button release, `u32` = button keycode
    MouseUp(u32),

    /// key pressed, `u32` = keycode
    KeyUp(u32),

    /// key released, `u32` = keycode, `bool` = holding
    KeyDown(u32, bool),

    /// mouse moved first `f32` on the x-axis and second `f32` on the y-axis
    MouseMove(f32, f32),

    /// focus changed, `bool` = window is focused
    FocusChange(bool),

    /// window is resized, `[i32;2]` are the new dimensions
    WindowResize([i32; 2]),

    /// event unique to window manager wich can not be defined with [`WindowEvent`]
    Undefined
}

use crate::engine::Engine;
/// trait implemented by engine's [`Engine::get_window`]
///
/// #### pub(crate) functions
///
/// I don't know how to private individual functions, so these functions should not be called
/// if you don't know what you're doing:
///
/// `init_connection`,
///
/// `init_window`,
///
/// `create_surface_khr`,
///
/// `commit_suicide`,
///
/// `supports_physical_device_queue`,
///
/// and `get_events` wich should be called trough [`Engine::get_events`]
///
pub trait Window {
    /// get's the width of the window
    fn get_width(&self) -> u32;

    /// get's the height of the window
    fn get_height(&self) -> u32;

    /// set's the width of the window
    fn set_width(&mut self, width: u32);

    /// set's the height of the window
    fn set_height(&mut self, height: u32);

    fn init_connection(dimensions: [i32; 2]) -> Self where Self: Sized;

    fn init_window(&mut self, name: &str);

    fn create_surface_khr(&self, instance: crate::vulkan::instance::VkInstance) -> VkSurfaceKHR;

    fn get_events(&mut self, dimensions: [i32; 2]) -> Vec<WindowEvent>;

    fn supports_physical_device_queue(&self, physical_device: VkPhysicalDevice, queue: u32) -> bool;

    fn commit_suicide(&self);

    /// moves the mouse to [x, y]
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
