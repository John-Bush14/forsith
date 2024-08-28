use crate::vulkan::{
    window::{
        Window,
        WindowEvent,
        VkSurfaceKHR,
        wayland::WayWindow
    },
    devices::physical_device::VkPhysicalDevice
};


impl Window for WayWindow {
    fn get_width(&self) -> u32 {todo!();}
    fn get_height(&self) -> u32 {todo!();}

    fn set_width(&mut self, _width: u32) {todo!();}
    fn set_height(&mut self, _height: u32) {todo!();}

    fn init_connection(_dimensions: [i32; 2]) -> Self where Self: Sized {return WayWindow {}}

    fn init_window(&mut self, _name: &str) {panic!("wayland not yet implemented!!!");}

    fn create_surface_khr(&self, _instance: crate::vulkan::instance::VkInstance) -> VkSurfaceKHR {todo!();}

    fn poll_events(&mut self, _dimensions: [i32; 2]) -> Vec<WindowEvent> {todo!();}

    fn supports_physical_device_queue(&self, _physical_device: VkPhysicalDevice, _queue: u32) -> bool {todo!();}

    fn commit_suicide(&self) {todo!();}

    fn set_mouse(&mut self, _x: f32, _y: f32) {todo!();}
}
