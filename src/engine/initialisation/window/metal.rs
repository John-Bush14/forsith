use crate::vulkan::{
    window::{
        Window,
        WindowEvent,
        VkSurfaceKHR,
        metal::MWindow
    },
    devices::physical_device::VkPhysicalDevice
};




impl Window for MWindow {
    fn get_width(&self) -> u32 {todo!();}
    fn get_height(&self) -> u32 {todo!();}

    fn set_width(&mut self, width: u32) {todo!();}
    fn set_height(&mut self, height: u32) {todo!();}

    fn init_connection(dimensions: [i32; 2]) -> Self where Self: Sized {return MWindow {}}

    fn init_window(&mut self, name: &str) {panic!("metal not yet implemented!!!")}

    fn create_surfaceKHR(&self, instance: crate::vulkan::instance::VkInstance) -> VkSurfaceKHR {todo!();}

    fn poll_events(&mut self, dimensions: [i32; 2]) -> Vec<WindowEvent> {todo!();}

    fn supports_physical_device_queue(&self, physical_device: VkPhysicalDevice, queue: u32) -> bool {todo!();}

    fn commit_suicide(&self) {todo!();}

    fn set_mouse(&mut self, x: f32, y: f32) {todo!();}
}
