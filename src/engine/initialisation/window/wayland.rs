use crate::vulkan::{
    devices::physical_device::VkPhysicalDevice, window::{
        wayland::{
            vkGetPhysicalDeviceWaylandPresentationSupportKHR, wl_display_connect, WayWindow
        }, VkSurfaceKHR, Window, WindowEvent
    }
};


use std::ffi::{c_char, c_void};


impl Window for WayWindow {
    fn get_width(&self) -> u32 {todo!();}
    fn get_height(&self) -> u32 {todo!();}

    fn set_width(&mut self, _width: u32) {todo!();}
    fn set_height(&mut self, _height: u32) {todo!();}

    fn init_connection(_dimensions: [i32; 2]) -> Self where Self: Sized {
        let display = unsafe {wl_display_connect(std::ptr::null())};

        if display.is_null() {panic!("wl_display_connect failed!");}


        return WayWindow {display};
    }

    fn init_window(&mut self, _name: &str) {panic!("wayland not yet implemented!!!");}

    fn create_surface_khr(&self, _instance: crate::vulkan::instance::VkInstance) -> VkSurfaceKHR {todo!();}

    fn poll_and_process_events(&mut self, _dimensions: [i32; 2]) -> Vec<WindowEvent> {todo!();}

    fn supports_physical_device_queue(&self, physical_device: VkPhysicalDevice, queue_family: u32) -> bool {
        return unsafe {vkGetPhysicalDeviceWaylandPresentationSupportKHR(physical_device, queue_family, self.display)} == 1;
    }

    fn commit_suicide(&self) {todo!();}

    fn set_mouse(&mut self, _x: f32, _y: f32) {todo!();}
}
