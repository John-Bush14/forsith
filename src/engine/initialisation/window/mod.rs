use crate::vulkan::{
    window::{
        Window,
        x11::{
            XWindow
        }
    }
};


mod x11;


impl super::super::Engine { pub fn create_window(name: String) -> Box<dyn Window> { unsafe {
    return Box::new(XWindow::init(name))
}}}

impl super::super::Engine { pub fn create_surface_KHR(&mut self, instance: crate::vulkan::instance::VkInstance) { unsafe {
    self.surface_khr = self.window.create_surfaceKHR(instance);
}}}
